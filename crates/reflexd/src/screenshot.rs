//! Asynchronous screenshot actor using grim and wl-copy.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::process::Command;
use tracing::info;

pub struct ScreenshotActor;

impl ScreenshotActor {
    /// Ensure screenshots directory exists (~/Pictures/Screenshots)
    fn get_screenshot_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(home).join("Pictures").join("Screenshots");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    /// Capture specified bounding box (x, y, w, h) asynchronously
    pub fn capture_box(x: i32, y: i32, w: i32, h: i32) {
        tokio::spawn(async move {
            let geom = format!("{},{} {}x{}", x, y, w, h);
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let filename = format!("screenshot_{}.png", timestamp);
            let filepath = Self::get_screenshot_dir().join(filename);

            info!("Capturing geometry box '{}' to {:?}", geom, filepath);

            // Execute grim and save to file
            let grim_status = Command::new("grim")
                .args(["-g", &geom, filepath.to_str().unwrap_or("")])
                .status()
                .await;

            if let Ok(status) = grim_status {
                if status.success() {
                    // Pipe file to wl-copy
                    if let Ok(file) = std::fs::File::open(&filepath) {
                        let _ = Command::new("wl-copy")
                            .args(["--type", "image/png"])
                            .stdin(file)
                            .status()
                            .await;
                    }
                }
            }
        });
    }

    /// Capture full screen asynchronously
    pub fn capture_fullscreen() {
        tokio::spawn(async move {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let filename = format!("screenshot_full_{}.png", timestamp);
            let filepath = Self::get_screenshot_dir().join(filename);

            info!("Capturing fullscreen to {:?}", filepath);

            let grim_status = Command::new("grim")
                .arg(filepath.to_str().unwrap_or(""))
                .status()
                .await;

            if let Ok(status) = grim_status {
                if status.success() {
                    if let Ok(file) = std::fs::File::open(&filepath) {
                        let _ = Command::new("wl-copy")
                            .args(["--type", "image/png"])
                            .stdin(file)
                            .status()
                            .await;
                    }
                }
            }
        });
    }

    /// Interactive selection with slurp
    pub fn interactive_snip() {
        tokio::spawn(async move {
            let slurp_output = Command::new("slurp").output().await;
            if let Ok(out) = slurp_output {
                if out.status.success() {
                    let geom = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !geom.is_empty() {
                        let timestamp = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let filename = format!("snip_{}.png", timestamp);
                        let filepath = Self::get_screenshot_dir().join(filename);

                        let grim_status = Command::new("grim")
                            .args(["-g", &geom, filepath.to_str().unwrap_or("")])
                            .status()
                            .await;

                        if let Ok(status) = grim_status {
                            if status.success() {
                                if let Ok(file) = std::fs::File::open(&filepath) {
                                    let _ = Command::new("wl-copy")
                                        .args(["--type", "image/png"])
                                        .stdin(file)
                                        .status()
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
