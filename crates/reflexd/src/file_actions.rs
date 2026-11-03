//! File actions driver: zip/unzip of selected files via Wayland clipboard / wtype.

use reflex_core::ast::TargetDirectory;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::info;

/// Decodes percent-encoded URL sequences (e.g. %20 -> space).
pub fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(byte) = u8::from_str_radix(
                    &format!("{}{}", h1 as char, h2 as char),
                    16,
                ) {
                    bytes.push(byte);
                    continue;
                }
            }
        }
        bytes.push(b);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

/// Resolves currently selected files in the active window.
///
/// 1. Emits a Ctrl+C keypress via `wtype` to prompt the active window (e.g. Dolphin) to copy the selection.
/// 2. Reads the Wayland clipboard for `text/uri-list`.
/// 3. Falls back to `wl-paste` (plain text) and `wl-paste --primary` if no URI list is present.
pub async fn resolve_selected_files() -> Result<Vec<PathBuf>, String> {
    // Pulse Ctrl+C via wtype to copy selection in active window
    let _ = tokio::process::Command::new("wtype")
        .args(["-M", "ctrl", "-k", "c", "-m", "ctrl"])
        .status()
        .await;

    // Small delay for clipboard synchronization
    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;

    // 1. Try text/uri-list (standard for Dolphin, Nautilus, Thunar, etc.)
    if let Ok(output) = tokio::process::Command::new("wl-paste")
        .args(["-t", "text/uri-list"])
        .output()
        .await
    {
        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout);
            let mut paths = Vec::new();
            for line in raw.lines() {
                let trimmed = line.trim();
                if let Some(uri) = trimmed.strip_prefix("file://") {
                    let decoded = percent_decode(uri);
                    let p = PathBuf::from(&decoded);
                    if p.exists() {
                        paths.push(p);
                    }
                }
            }
            if !paths.is_empty() {
                return Ok(paths);
            }
        }
    }

    // 2. Fallback: try plain clipboard text
    if let Ok(output) = tokio::process::Command::new("wl-paste")
        .output()
        .await
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let decoded = if let Some(uri) = text.strip_prefix("file://") {
                percent_decode(uri)
            } else {
                text
            };
            let p = PathBuf::from(&decoded);
            if p.exists() {
                return Ok(vec![p]);
            }
        }
    }

    // 3. Fallback: try primary selection (e.g. highlighted text in Kitty / terminal)
    if let Ok(output) = tokio::process::Command::new("wl-paste")
        .args(["--primary"])
        .output()
        .await
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let p = PathBuf::from(&text);
            if p.exists() {
                return Ok(vec![p]);
            }
        }
    }

    Err("No selected file detected. Highlight or select a file in Dolphin or terminal first.".to_string())
}

/// Zips the selected files/directories into an archive in the parent directory.
pub async fn zip_selected(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No files specified for zipping.".to_string());
    }

    if paths.len() == 1 {
        let target = &paths[0];
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        let file_name = target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("target");
        let stem = target
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archive");

        let mut archive_name = format!("{}.zip", stem);
        let mut archive_path = parent.join(&archive_name);

        if archive_path == *target {
            archive_name = format!("{}_archive.zip", stem);
            archive_path = parent.join(&archive_name);
        }

        let mut counter = 1;
        while archive_path.exists() {
            archive_name = format!("{}_{}.zip", stem, counter);
            archive_path = parent.join(&archive_name);
            counter += 1;
        }

        let output = tokio::process::Command::new("zip")
            .args(["-r", "-q", &archive_name, file_name])
            .current_dir(parent)
            .output()
            .await
            .map_err(|e| format!("Failed to spawn zip: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("zip error: {}", err.trim()));
        }

        info!("Zipped '{}' to '{}'", target.display(), archive_path.display());
        Ok(format!("Zipped '{}' to '{}'", file_name, archive_name))
    } else {
        // Multiple paths: find common parent
        let parent = paths[0].parent().unwrap_or_else(|| Path::new("."));
        let mut archive_name = "archive.zip".to_string();
        let mut archive_path = parent.join(&archive_name);
        let mut counter = 1;
        while archive_path.exists() {
            archive_name = format!("archive_{}.zip", counter);
            archive_path = parent.join(&archive_name);
            counter += 1;
        }

        let mut cmd = tokio::process::Command::new("zip");
        cmd.args(["-r", "-q", &archive_name]);
        for p in paths {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                cmd.arg(name);
            }
        }
        cmd.current_dir(parent);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to spawn zip: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("zip error: {}", err.trim()));
        }

        info!("Zipped {} files to '{}'", paths.len(), archive_path.display());
        Ok(format!("Zipped {} items to '{}'", paths.len(), archive_name))
    }
}

/// Unzips or extracts the selected archive.
pub async fn unzip_selected(paths: &[PathBuf], into_subfolder: bool) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No archive specified for extraction.".to_string());
    }

    let archive = &paths[0];
    if !archive.exists() {
        return Err(format!("Archive not found: {}", archive.display()));
    }

    let parent = archive.parent().unwrap_or_else(|| Path::new("."));
    let file_name = archive
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");
    let name_lower = file_name.to_lowercase();

    let target_dir = if into_subfolder {
        let stem = if name_lower.ends_with(".tar.gz") {
            &file_name[..file_name.len() - 7]
        } else if name_lower.ends_with(".tar.xz") || name_lower.ends_with(".tar.bz2") {
            &file_name[..file_name.len() - 8]
        } else {
            archive.file_stem().and_then(|s| s.to_str()).unwrap_or("extracted")
        };

        let mut dir = parent.join(stem);
        let mut counter = 1;
        while dir.exists() {
            dir = parent.join(format!("{}_{}", stem, counter));
            counter += 1;
        }
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
        dir
    } else {
        parent.to_path_buf()
    };

    if name_lower.ends_with(".zip") {
        let output = tokio::process::Command::new("unzip")
            .args(["-q", "-o", archive.to_str().unwrap(), "-d", target_dir.to_str().unwrap()])
            .output()
            .await
            .map_err(|e| format!("Failed to spawn unzip: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("unzip error: {}", err.trim()));
        }
    } else if name_lower.ends_with(".tar.gz")
        || name_lower.ends_with(".tgz")
        || name_lower.ends_with(".tar.xz")
        || name_lower.ends_with(".tar.bz2")
        || name_lower.ends_with(".tar")
    {
        let output = tokio::process::Command::new("tar")
            .args(["-xf", archive.to_str().unwrap(), "-C", target_dir.to_str().unwrap()])
            .output()
            .await
            .map_err(|e| format!("Failed to spawn tar: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tar error: {}", err.trim()));
        }
    } else {
        return Err(format!(
            "Unsupported archive format for '{}'. Expected .zip, .tar.gz, .tgz, .tar.xz",
            file_name
        ));
    }

    if into_subfolder {
        let dest_name = target_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("folder");
        info!("Extracted '{}' to '{}/'", file_name, dest_name);
        Ok(format!("Extracted '{}' to '{}/'", file_name, dest_name))
    } else {
        info!("Extracted '{}' in-place", file_name);
        Ok(format!("Extracted '{}' in-place", file_name))
    }
}

/// Helper: Pipes raw bytes into wl-copy with optional mime-type.
pub async fn write_to_wl_copy(data: &[u8], mime_type: Option<&str>) -> Result<(), String> {
    let mut cmd = tokio::process::Command::new("wl-copy");
    if let Some(mime) = mime_type {
        cmd.args(["-t", mime]);
    }
    cmd.stdin(std::process::Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn wl-copy: {}", e))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(data)
            .await
            .map_err(|e| format!("Failed to write to wl-copy: {}", e))?;
    }
    let status = child
        .wait()
        .await
        .map_err(|e| format!("wl-copy error: {}", e))?;
    if !status.success() {
        return Err("wl-copy exited with error status".to_string());
    }
    Ok(())
}

/// Helper: Formats current date/time using the local system `date` utility.
pub async fn get_current_timestamp(format_str: &str) -> String {
    if let Ok(output) = tokio::process::Command::new("date")
        .arg(format_str)
        .output()
        .await
    {
        if output.status.success() {
            let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !res.is_empty() {
                return res;
            }
        }
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

/// Helper: Resolves well-known XDG target directories.
pub fn resolve_target_directory(dir: TargetDirectory) -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/itshimelz"));
    match dir {
        TargetDirectory::Downloads => home.join("Downloads"),
        TargetDirectory::Documents => home.join("Documents"),
        TargetDirectory::Desktop => home.join("Desktop"),
        TargetDirectory::Pictures => home.join("Pictures"),
        TargetDirectory::Videos => home.join("Videos"),
        TargetDirectory::Music => home.join("Music"),
    }
}

/// Copies full paths or filenames of selected files to the Wayland clipboard.
pub async fn copy_path(paths: &[PathBuf], full: bool) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let text = if full {
        paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        paths
            .iter()
            .map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    write_to_wl_copy(text.as_bytes(), None).await?;
    let label = if full { "path" } else { "filename" };
    Ok(format!("Copied {} of {} item(s)", label, paths.len()))
}

/// Copies the text content of the selected file to clipboard.
pub async fn copy_contents(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let meta = tokio::fs::metadata(target)
        .await
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;
    if meta.len() > 10 * 1024 * 1024 {
        return Err("File too large (>10MB) to copy to clipboard".to_string());
    }
    let content = tokio::fs::read_to_string(target)
        .await
        .map_err(|_| "Selected file is binary or not UTF-8 text".to_string())?;

    write_to_wl_copy(content.as_bytes(), None).await?;
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    Ok(format!("Copied contents of '{}' ({} bytes)", name, content.len()))
}

/// Copies the selected image to the Wayland clipboard as image/png.
pub async fn copy_image(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");

    let magick_out = tokio::process::Command::new("magick")
        .args([target.to_str().unwrap(), "png:-"])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn magick: {}", e))?;

    if !magick_out.status.success() {
        let err = String::from_utf8_lossy(&magick_out.stderr);
        return Err(format!("ImageMagick error: {}", err.trim()));
    }

    write_to_wl_copy(&magick_out.stdout, Some("image/png")).await?;
    Ok(format!("Copied image '{}' to clipboard", name))
}

/// Saves current Wayland clipboard contents into the active directory.
pub async fn save_clipboard_here(paths: &[PathBuf]) -> Result<String, String> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/itshimelz"));
    let target_dir = if !paths.is_empty() {
        let p = &paths[0];
        if p.is_dir() {
            p.clone()
        } else {
            p.parent().unwrap_or(&home).to_path_buf()
        }
    } else {
        home.join("Downloads")
    };

    let timestamp = get_current_timestamp("+%Y%m%d_%H%M%S").await;

    // Check if clipboard has image
    if let Ok(img_out) = tokio::process::Command::new("wl-paste")
        .args(["-t", "image/png"])
        .output()
        .await
    {
        if img_out.status.success() && !img_out.stdout.is_empty() {
            let file_name = format!("clipboard_{}.png", timestamp);
            let dest = target_dir.join(&file_name);
            tokio::fs::write(&dest, &img_out.stdout)
                .await
                .map_err(|e| format!("Failed to write clipboard image: {}", e))?;
            return Ok(format!("Saved clipboard image to '{}'", file_name));
        }
    }

    // Fallback: check if clipboard has text
    if let Ok(txt_out) = tokio::process::Command::new("wl-paste")
        .output()
        .await
    {
        if txt_out.status.success() && !txt_out.stdout.is_empty() {
            let file_name = format!("clipboard_{}.txt", timestamp);
            let dest = target_dir.join(&file_name);
            tokio::fs::write(&dest, &txt_out.stdout)
                .await
                .map_err(|e| format!("Failed to write clipboard text: {}", e))?;
            return Ok(format!("Saved clipboard text to '{}'", file_name));
        }
    }

    Err("Clipboard is empty or has unsupported format".to_string())
}

/// Duplicates selected files or directories with auto-incrementing suffix.
pub async fn duplicate_selected(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected to duplicate".to_string());
    }

    let mut count = 0;
    for p in paths {
        let parent = p.parent().unwrap_or_else(|| Path::new("."));
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("copy");
        let ext = p.extension().and_then(|e| e.to_str());

        let mut target_name = if let Some(e) = ext {
            format!("{}_copy.{}", stem, e)
        } else {
            format!("{}_copy", stem)
        };
        let mut target_path = parent.join(&target_name);

        let mut counter = 1;
        while target_path.exists() {
            target_name = if let Some(e) = ext {
                format!("{}_copy_{}.{}", stem, counter, e)
            } else {
                format!("{}_copy_{}", stem, counter)
            };
            target_path = parent.join(&target_name);
            counter += 1;
        }

        if p.is_dir() {
            let status = tokio::process::Command::new("cp")
                .args(["-r", p.to_str().unwrap(), target_path.to_str().unwrap()])
                .status()
                .await
                .map_err(|e| format!("Failed to copy directory: {}", e))?;
            if !status.success() {
                return Err("Failed to duplicate directory".to_string());
            }
        } else {
            tokio::fs::copy(p, &target_path)
                .await
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
        count += 1;
    }

    Ok(format!("Duplicated {} item(s)", count))
}

/// Moves selected files to Freedesktop Trash with compliant `.trashinfo` metadata.
pub async fn trash_selected(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected to trash".to_string());
    }

    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/itshimelz"));
    let trash_files = home.join(".local/share/Trash/files");
    let trash_info = home.join(".local/share/Trash/info");
    tokio::fs::create_dir_all(&trash_files)
        .await
        .map_err(|e| format!("Failed to create trash directory: {}", e))?;
    tokio::fs::create_dir_all(&trash_info)
        .await
        .map_err(|e| format!("Failed to create trash info directory: {}", e))?;

    let del_date = get_current_timestamp("+%Y-%m-%dT%H:%M:%S").await;
    let mut count = 0;

    for p in paths {
        let file_name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("trashed_item");

        let mut dest_name = file_name.to_string();
        let mut dest_path = trash_files.join(&dest_name);
        let mut info_path = trash_info.join(format!("{}.trashinfo", dest_name));

        let mut counter = 1;
        while dest_path.exists() || info_path.exists() {
            dest_name = format!("{}.{}", file_name, counter);
            dest_path = trash_files.join(&dest_name);
            info_path = trash_info.join(format!("{}.trashinfo", dest_name));
            counter += 1;
        }

        let abs_orig = tokio::fs::canonicalize(p)
            .await
            .unwrap_or_else(|_| p.clone());
        let info_content = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            abs_orig.display(),
            del_date
        );

        tokio::fs::write(&info_path, info_content)
            .await
            .map_err(|e| format!("Failed to write trashinfo: {}", e))?;

        if tokio::fs::rename(p, &dest_path).await.is_err() {
            // Fallback for cross-device files
            if p.is_dir() {
                let _ = tokio::process::Command::new("cp")
                    .args(["-r", p.to_str().unwrap(), dest_path.to_str().unwrap()])
                    .status()
                    .await;
                let _ = tokio::fs::remove_dir_all(p).await;
            } else {
                let _ = tokio::fs::copy(p, &dest_path).await;
                let _ = tokio::fs::remove_file(p).await;
            }
        }
        count += 1;
    }

    Ok(format!("Moved {} item(s) to Trash", count))
}

/// Moves selected files to the target standard directory.
pub async fn move_to_directory(
    paths: &[PathBuf],
    target_dir_type: TargetDirectory,
) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected to move".to_string());
    }

    let target_dir = resolve_target_directory(target_dir_type);
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    let mut count = 0;
    for p in paths {
        let file_name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("item");
        let mut dest_name = file_name.to_string();
        let mut dest_path = target_dir.join(&dest_name);

        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("item");
        let ext = p.extension().and_then(|e| e.to_str());

        let mut counter = 1;
        while dest_path.exists() {
            dest_name = if let Some(e) = ext {
                format!("{}_{}.{}", stem, counter, e)
            } else {
                format!("{}_{}", stem, counter)
            };
            dest_path = target_dir.join(&dest_name);
            counter += 1;
        }

        if tokio::fs::rename(p, &dest_path).await.is_err() {
            // Cross-device fallback
            if p.is_dir() {
                let _ = tokio::process::Command::new("cp")
                    .args(["-r", p.to_str().unwrap(), dest_path.to_str().unwrap()])
                    .status()
                    .await;
                let _ = tokio::fs::remove_dir_all(p).await;
            } else {
                let _ = tokio::fs::copy(p, &dest_path).await;
                let _ = tokio::fs::remove_file(p).await;
            }
        }
        count += 1;
    }

    let dir_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("target");
    Ok(format!("Moved {} item(s) to {}", count, dir_name))
}

/// Creates a new directory in the active folder.
pub async fn new_folder(paths: &[PathBuf], name: &str) -> Result<String, String> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/itshimelz"));
    let parent = if !paths.is_empty() {
        let p = &paths[0];
        if p.is_dir() {
            p.clone()
        } else {
            p.parent().unwrap_or(&home).to_path_buf()
        }
    } else {
        home.join("Downloads")
    };

    let mut folder_name = name.to_string();
    let mut dest_path = parent.join(&folder_name);
    let mut counter = 1;
    while dest_path.exists() {
        folder_name = format!("{}_{}", name, counter);
        dest_path = parent.join(&folder_name);
        counter += 1;
    }

    tokio::fs::create_dir_all(&dest_path)
        .await
        .map_err(|e| format!("Failed to create folder: {}", e))?;
    Ok(format!("Created folder '{}'", folder_name))
}

/// Renames the selected file, preserving original extension if not specified.
pub async fn rename_selected(paths: &[PathBuf], new_name: &str) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected to rename".to_string());
    }
    let target = &paths[0];
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let old_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let final_name = if new_name.contains('.') {
        new_name.to_string()
    } else if let Some(ext) = target.extension().and_then(|e| e.to_str()) {
        format!("{}.{}", new_name, ext)
    } else {
        new_name.to_string()
    };

    let dest = parent.join(&final_name);
    tokio::fs::rename(target, &dest)
        .await
        .map_err(|e| format!("Failed to rename: {}", e))?;
    Ok(format!("Renamed '{}' to '{}'", old_name, final_name))
}

/// Prepends current YYYY-MM-DD date stamp to the selected filename.
pub async fn timestamp_selected(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected to timestamp".to_string());
    }
    let date_str = get_current_timestamp("+%Y-%m-%d").await;
    let mut count = 0;

    for p in paths {
        let parent = p.parent().unwrap_or_else(|| Path::new("."));
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        if name.starts_with(&date_str) {
            continue;
        }
        let target_name = format!("{}_{}", date_str, name);
        let dest = parent.join(&target_name);
        tokio::fs::rename(p, &dest)
            .await
            .map_err(|e| format!("Failed to timestamp: {}", e))?;
        count += 1;
    }

    Ok(format!("Timestamped {} file(s)", count))
}

/// Cleans filename: lowercases, converts whitespace/symbols to hyphens, collapses dashes.
pub async fn clean_filename(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected to clean".to_string());
    }
    let mut count = 0;

    for p in paths {
        let parent = p.parent().unwrap_or_else(|| Path::new("."));
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = p.extension().and_then(|e| e.to_str());

        let mut cleaned = String::new();
        let mut last_was_dash = false;
        for c in stem.chars() {
            if c.is_ascii_alphanumeric() {
                cleaned.push(c.to_ascii_lowercase());
                last_was_dash = false;
            } else if c == '.' || c == '_' {
                cleaned.push(c);
                last_was_dash = false;
            } else if !last_was_dash {
                cleaned.push('-');
                last_was_dash = true;
            }
        }
        let cleaned_stem = cleaned.trim_matches('-').trim_matches('_');
        let final_stem = if cleaned_stem.is_empty() { "file" } else { cleaned_stem };

        let final_name = if let Some(e) = ext {
            format!("{}.{}", final_stem, e.to_lowercase())
        } else {
            final_stem.to_string()
        };

        let dest = parent.join(&final_name);
        if dest != *p {
            tokio::fs::rename(p, &dest)
                .await
                .map_err(|e| format!("Failed to clean filename: {}", e))?;
            count += 1;
        }
    }

    Ok(format!("Cleaned {} filename(s)", count))
}

/// Strips EXIF metadata using ImageMagick (images) or FFmpeg (media).
pub async fn strip_metadata(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let mut count = 0;

    for p in paths {
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "tiff" | "gif" => {
                let status = tokio::process::Command::new("magick")
                    .args([p.to_str().unwrap(), "-strip", p.to_str().unwrap()])
                    .status()
                    .await
                    .map_err(|e| format!("Failed to run magick: {}", e))?;
                if status.success() {
                    count += 1;
                }
            }
            "mp4" | "mkv" | "mov" | "mp3" | "m4a" | "flac" | "wav" | "webm" => {
                let parent = p.parent().unwrap_or_else(|| Path::new("."));
                let temp = parent.join(format!(
                    ".tmp_strip_{}",
                    p.file_name().unwrap().to_str().unwrap()
                ));
                let status = tokio::process::Command::new("ffmpeg")
                    .args([
                        "-y",
                        "-i",
                        p.to_str().unwrap(),
                        "-map_metadata",
                        "-1",
                        "-c",
                        "copy",
                        temp.to_str().unwrap(),
                    ])
                    .status()
                    .await
                    .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;
                if status.success() {
                    let _ = tokio::fs::rename(&temp, p).await;
                    count += 1;
                } else {
                    let _ = tokio::fs::remove_file(&temp).await;
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported file format for metadata stripping: '{}'",
                    ext
                ));
            }
        }
    }

    Ok(format!("Stripped metadata from {} file(s)", count))
}

/// Extracts audio track from video/audio using FFmpeg.
pub async fn extract_audio(paths: &[PathBuf], format: &str) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");

    let out_name = format!("{}.{}", stem, format);
    let mut out_path = parent.join(&out_name);
    let mut counter = 1;
    while out_path.exists() {
        out_path = parent.join(format!("{}_{}.{}", stem, counter, format));
        counter += 1;
    }

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args(["-y", "-i", target.to_str().unwrap(), "-vn"]);
    match format {
        "mp3" => {
            cmd.args(["-c:a", "libmp3lame", "-q:a", "2"]);
        }
        "wav" => {
            cmd.args(["-c:a", "pcm_s16le"]);
        }
        "flac" => {
            cmd.args(["-c:a", "flac"]);
        }
        "m4a" => {
            cmd.args(["-c:a", "aac", "-b:a", "192k"]);
        }
        _ => {
            cmd.args(["-c:a", "copy"]);
        }
    }
    cmd.arg(out_path.to_str().unwrap());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg error: {}", err.trim()));
    }

    let final_name = out_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&out_name);
    Ok(format!("Extracted audio to '{}'", final_name))
}

/// Converts image format using ImageMagick.
pub async fn convert_image(paths: &[PathBuf], format: &str) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let out_name = format!("{}.{}", stem, format);
    let mut out_path = parent.join(&out_name);
    let mut counter = 1;
    while out_path.exists() {
        out_path = parent.join(format!("{}_{}.{}", stem, counter, format));
        counter += 1;
    }

    let output = tokio::process::Command::new("magick")
        .args([target.to_str().unwrap(), out_path.to_str().unwrap()])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn magick: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ImageMagick error: {}", err.trim()));
    }

    let final_name = out_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&out_name);
    Ok(format!("Converted to '{}'", final_name))
}

/// Creates a palette-optimized GIF from video using FFmpeg.
pub async fn make_gif(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("animation");

    let out_name = format!("{}.gif", stem);
    let mut out_path = parent.join(&out_name);
    let mut counter = 1;
    while out_path.exists() {
        out_path = parent.join(format!("{}_{}.gif", stem, counter));
        counter += 1;
    }

    let vf = "fps=15,scale=640:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse";
    let output = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            target.to_str().unwrap(),
            "-vf",
            vf,
            out_path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg error: {}", err.trim()));
    }

    let final_name = out_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&out_name);
    Ok(format!("Created GIF '{}'", final_name))
}

/// Extracts PDF pages into high-resolution PNG images via pdftoppm.
pub async fn pdf_to_images(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("doc");

    let prefix = parent.join(format!("{}_page", stem));
    let output = tokio::process::Command::new("pdftoppm")
        .args([
            "-png",
            "-r",
            "150",
            target.to_str().unwrap(),
            prefix.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn pdftoppm: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pdftoppm error: {}", err.trim()));
    }

    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document");
    Ok(format!("Extracted pages from '{}' to PNG images", file_name))
}

/// Returns summary info (size, permissions, kind) for the selected item.
pub async fn file_info(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let meta = tokio::fs::metadata(target)
        .await
        .map_err(|e| format!("Failed to read metadata: {}", e))?;

    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("item");
    let kind = if meta.is_dir() { "Directory" } else { "File" };
    let size_bytes = meta.len();
    let size_fmt = if size_bytes < 1024 {
        format!("{} B", size_bytes)
    } else if size_bytes < 1024 * 1024 {
        format!("{:.1} KB", size_bytes as f64 / 1024.0)
    } else if size_bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    };

    Ok(format!("{}: {} ({})", name, size_fmt, kind))
}

/// Calculates SHA-256 checksum and copies it to clipboard.
pub async fn checksum_selected(paths: &[PathBuf]) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No file selected".to_string());
    }
    let target = &paths[0];
    let output = tokio::process::Command::new("sha256sum")
        .arg(target.to_str().unwrap())
        .output()
        .await
        .map_err(|e| format!("Failed to run sha256sum: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("sha256sum error: {}", err.trim()));
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    let hash = out_str
        .split_whitespace()
        .next()
        .ok_or_else(|| "Failed to parse sha256 output".to_string())?;

    write_to_wl_copy(hash.as_bytes(), None).await?;
    let prefix = if hash.len() > 12 { &hash[..12] } else { hash };
    Ok(format!("SHA-256: {}... (copied to clipboard)", prefix))
}

/// Opens the Kitty terminal in the directory of the selected file.
pub async fn open_in_terminal(paths: &[PathBuf]) -> Result<String, String> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/itshimelz"));
    let target_dir = if !paths.is_empty() {
        let p = &paths[0];
        if p.is_dir() {
            p.clone()
        } else {
            p.parent().unwrap_or(&home).to_path_buf()
        }
    } else {
        home
    };

    let _ = tokio::process::Command::new("kitty")
        .args(["--directory", target_dir.to_str().unwrap()])
        .spawn()
        .map_err(|e| format!("Failed to spawn kitty: {}", e))?;

    let name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("folder");
    Ok(format!("Opened terminal in '{}'", name))
}
