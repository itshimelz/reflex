//! File actions driver: zip/unzip of selected files via Wayland clipboard / wtype.

use std::path::{Path, PathBuf};
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
