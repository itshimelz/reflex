//! High-speed non-blocking broadcast IPC server for Reflex HUD.

use reflex_core::hud::{HudEvent, HudIcon, HudTone};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tracing::{info, warn};

pub struct HudBroadcaster {
    tx: broadcast::Sender<HudEvent>,
}

impl Default for HudBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl HudBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self { tx }
    }

    /// Emit an event to all connected HUD pill clients (0μs, non-blocking)
    pub fn emit(&self, event: HudEvent) {
        let _ = self.tx.send(event);
    }

    pub fn sender(&self) -> broadcast::Sender<HudEvent> {
        self.tx.clone()
    }

    /// Convenient helper mapping legacy notification summaries and icons to HUD events
    pub fn notify(&self, summary: &str, body: &str, icon: &str, timeout_ms: i32) {
        let hud_icon = match icon {
            "audio-volume-high" | "audio-volume-medium" => HudIcon::Volume2,
            "audio-volume-muted" => HudIcon::VolumeX,
            "audio-headphones" => HudIcon::Headphones,
            "dialog-warning" => HudIcon::ShieldAlert,
            "dialog-error" => HudIcon::AlertTriangle,
            "window-close" => HudIcon::Check,
            "edit-clear" => HudIcon::Clipboard,
            "camera-photo" => HudIcon::Camera,
            "system-run" => HudIcon::Zap,
            _ => {
                if summary.contains("Clamped") || summary.contains("Warning") {
                    HudIcon::ShieldAlert
                } else if summary.contains("Close") {
                    HudIcon::Lock
                } else if summary.contains("Workspace") {
                    HudIcon::Layers
                } else {
                    HudIcon::Check
                }
            }
        };

        let hud_tone = match hud_icon {
            HudIcon::ShieldAlert => HudTone::Amber,
            HudIcon::AlertTriangle => HudTone::Coral,
            HudIcon::Lock => HudTone::Coral,
            HudIcon::Layers | HudIcon::Move | HudIcon::Zap => HudTone::Cyan,
            HudIcon::Sparkles => HudTone::Purple,
            _ => HudTone::Emerald,
        };

        let detail = if body.trim().is_empty() {
            None
        } else {
            Some(body.to_string())
        };

        let event = HudEvent {
            icon: hud_icon,
            title: summary.to_string(),
            detail,
            tone: hud_tone,
            hold_ms: if timeout_ms > 0 { Some(timeout_ms as u64) } else { None },
        };

        self.emit(event);
    }
}

pub async fn run_hud_server(
    broadcaster: Arc<HudBroadcaster>,
    socket_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = std::fs::remove_file(&socket_path);
    if let Some(parent) = socket_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let listener = UnixListener::bind(&socket_path)?;
    info!(path = %socket_path.display(), "Reflex HUD broadcast socket listening");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let broadcaster = Arc::clone(&broadcaster);
                tokio::spawn(async move {
                    handle_hud_client(stream, broadcaster).await;
                });
            }
            Err(e) => {
                warn!(error = %e, "Error accepting HUD client connection");
            }
        }
    }
}

async fn handle_hud_client(stream: UnixStream, broadcaster: Arc<HudBroadcaster>) {
    let (reader, mut writer) = stream.into_split();
    let mut rx = broadcaster.sender().subscribe();

    // 1. Stream broadcast events out to the HUD client
    let write_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                let payload = format!("{}\n", json);
                if writer.write_all(payload.as_bytes()).await.is_err() {
                    break;
                }
                if writer.flush().await.is_err() {
                    break;
                }
            }
        }
    });

    // 2. Accept events into the socket from scripts (e.g. reflex-agent-fallback)
    let read_task = tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        while buf_reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
            if let Ok(event) = serde_json::from_str::<HudEvent>(line.trim()) {
                broadcaster.emit(event);
            }
            line.clear();
        }
    });

    tokio::select! {
        _ = write_task => {},
        _ = read_task => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_hud_broadcaster_emission() {
        let broadcaster = HudBroadcaster::new();
        let mut rx = broadcaster.sender().subscribe();

        broadcaster.emit(HudEvent::new(HudIcon::Volume2, "Volume 40%", HudTone::Emerald));

        let received = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .expect("timeout")
            .expect("received");

        assert_eq!(received.icon, HudIcon::Volume2);
        assert_eq!(received.title, "Volume 40%");
        assert_eq!(received.tone, HudTone::Emerald);
    }

    #[tokio::test]
    async fn test_hud_broadcaster_notify_adapter() {
        let broadcaster = HudBroadcaster::new();
        let mut rx = broadcaster.sender().subscribe();

        broadcaster.notify("Workspace 3", "", "window-new", 2000);

        let received = tokio::time::timeout(Duration::from_millis(200), rx.recv())
            .await
            .expect("timeout")
            .expect("received");

        assert_eq!(received.icon, HudIcon::Layers);
        assert_eq!(received.title, "Workspace 3");
        assert_eq!(received.tone, HudTone::Cyan);
    }

    #[tokio::test]
    async fn test_hud_socket_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let sock = dir.path().join("hud-test.sock");
        let broadcaster = Arc::new(HudBroadcaster::new());

        let b_clone = Arc::clone(&broadcaster);
        let s_path = sock.clone();
        tokio::spawn(async move {
            let _ = run_hud_server(b_clone, s_path).await;
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let stream = UnixStream::connect(&sock).await.expect("connect");
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        // 1. Send event into the socket
        let event = HudEvent::new(HudIcon::Sparkles, "AI Thinking", HudTone::Purple).with_detail("testing");
        let json = format!("{}\n", serde_json::to_string(&event).unwrap());
        writer.write_all(json.as_bytes()).await.expect("write");
        writer.flush().await.expect("flush");

        // 2. Client stream should receive broadcast back
        let mut line = String::new();
        tokio::time::timeout(Duration::from_millis(300), buf_reader.read_line(&mut line))
            .await
            .expect("timeout")
            .expect("read line");

        let decoded: HudEvent = serde_json::from_str(line.trim()).expect("deserialize");
        assert_eq!(decoded.icon, HudIcon::Sparkles);
        assert_eq!(decoded.title, "AI Thinking");
        assert_eq!(decoded.detail.as_deref(), Some("testing"));
        assert_eq!(decoded.tone, HudTone::Purple);
    }
}
