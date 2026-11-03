//! Local LAN network listener for remote mobile/desktop automation.
//!
//! Listens on `0.0.0.0:9421` and accepts:
//! 1. HTTP REST requests:
//!    - `GET /ping` or `GET /api/status` -> 200 OK `{"status":"ok","app":"reflex","version":"1.1.1","context":{...}}`
//!    - `GET /api/context` -> 200 OK with live DesktopContextPayload (active window, media, volume)
//!    - `POST /api/command` -> executes command, returns JSON `IpcResponse`
//!    - `OPTIONS *` -> CORS headers
//! 2. Persistent Raw TCP streams (sub-5ms latency, `TCP_NODELAY`):
//!    - `PING\n` -> replies with `PONG\n`
//!    - `CONTEXT\n` -> replies with JSON DesktopContextPayload
//!    - `COMMAND <text>\n` or `{"command":"<text>"}\n` -> executes and replies with JSON `IpcResponse`

use crate::server::{process_transcript, ServerContext};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

pub const DEFAULT_NETWORK_PORT: u16 = 9421;

#[derive(Deserialize)]
struct CommandPayload {
    command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopContextPayload {
    pub active_window: Option<WindowInfo>,
    pub media: Option<MediaInfo>,
    pub volume: Option<VolumeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub class: String,
    pub title: String,
    pub app_name: String,
    pub workspace: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub player: String,
    pub title: String,
    pub artist: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub percent: u8,
    pub muted: bool,
}

pub async fn fetch_desktop_context(ctx: &ServerContext) -> DesktopContextPayload {
    // 1. Fetch active window from Hyprland driver
    let active_window = if let Some(ref hl) = ctx.hyprland {
        match hl.fetch_active_window().await {
            Ok(Some(w)) => {
                let class_lower = w.class.to_lowercase();
                let title_lower = w.title.to_lowercase();
                let app_name = if class_lower.contains("dolphin")
                    || class_lower.contains("nautilus")
                    || class_lower.contains("thunar")
                {
                    "dolphin".to_string()
                } else if class_lower.contains("brave")
                    || class_lower.contains("chrome")
                    || class_lower.contains("firefox")
                {
                    if title_lower.contains("youtube") {
                        "youtube".to_string()
                    } else {
                        "browser".to_string()
                    }
                } else if class_lower.contains("spotify") || title_lower.contains("spotify") {
                    "spotify".to_string()
                } else if class_lower.contains("kitty")
                    || class_lower.contains("alacritty")
                    || class_lower.contains("foot")
                    || class_lower.contains("terminal")
                {
                    "terminal".to_string()
                } else if class_lower.contains("code")
                    || class_lower.contains("nvim")
                    || class_lower.contains("antigravity")
                    || class_lower.contains("studio")
                    || class_lower.contains("idea")
                {
                    "editor".to_string()
                } else {
                    w.class.split('-').next().unwrap_or(&w.class).to_string()
                };

                Some(WindowInfo {
                    class: w.class,
                    title: w.title,
                    app_name,
                    workspace: w.workspace,
                })
            }
            _ => None,
        }
    } else {
        None
    };

    // 2. Fetch media info via playerctl
    let media = match tokio::process::Command::new("playerctl")
        .args(["metadata", "--format", "{{title}}|||{{artist}}|||{{status}}|||{{playerName}}"])
        .output()
        .await
    {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let parts: Vec<&str> = s.split("|||").collect();
            if parts.len() >= 4 {
                Some(MediaInfo {
                    title: parts[0].trim().to_string(),
                    artist: parts[1].trim().to_string(),
                    status: parts[2].trim().to_string(),
                    player: parts[3].trim().to_string(),
                })
            } else if !s.is_empty() {
                Some(MediaInfo {
                    title: s,
                    artist: String::new(),
                    status: "Playing".to_string(),
                    player: "media".to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    };

    // 3. Fetch volume info via wpctl
    let volume = match tokio::process::Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .await
    {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut percent = 50u8;
            let mut muted = false;
            for token in s.split_whitespace() {
                if let Ok(v) = token.parse::<f32>() {
                    percent = ((v * 100.0).round() as u8).min(100);
                } else if token.contains("MUTED") {
                    muted = true;
                }
            }
            Some(VolumeInfo { percent, muted })
        }
        _ => None,
    };

    DesktopContextPayload {
        active_window,
        media,
        volume,
    }
}

pub async fn run_network_server(
    ctx: Arc<ServerContext>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("reflexd LAN network listener bound to http://{}", addr);
            l
        }
        Err(e) => {
            error!("Failed to bind network listener on {}: {}", addr, e);
            return Err(e.into());
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, _peer_addr)) => {
                let _ = stream.set_nodelay(true);
                let ctx_clone = Arc::clone(&ctx);
                tokio::spawn(async move {
                    handle_network_client(stream, ctx_clone).await;
                });
            }
            Err(e) => {
                warn!("Accept error on network listener: {}", e);
            }
        }
    }
}

async fn handle_network_client(mut stream: TcpStream, ctx: Arc<ServerContext>) {
    let mut peek_buf = [0u8; 8];
    let bytes_peeked = match stream.peek(&mut peek_buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let peek_str = String::from_utf8_lossy(&peek_buf[..bytes_peeked]);

    // Check if it's an HTTP request
    if peek_str.starts_with("GET ")
        || peek_str.starts_with("POST ")
        || peek_str.starts_with("OPTIONS ")
    {
        let mut buffer = [0u8; 8192];
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(n) if n > 0 => n,
            _ => return,
        };
        let raw_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        handle_http_request(&mut stream, &raw_data, &ctx).await;
    } else {
        // Persistent raw TCP streaming connection with sub-5ms latency
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.eq_ignore_ascii_case("ping") {
                let _ = writer.write_all(b"PONG\n").await;
                let _ = writer.flush().await;
            } else if trimmed.eq_ignore_ascii_case("context") {
                let context_data = fetch_desktop_context(&ctx).await;
                let json = serde_json::to_string(&context_data).unwrap_or_default();
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                let _ = writer.flush().await;
            } else {
                let cmd = if let Ok(payload) = serde_json::from_str::<CommandPayload>(trimmed) {
                    payload.command
                } else if trimmed.starts_with("COMMAND ") {
                    trimmed.trim_start_matches("COMMAND ").trim().to_string()
                } else {
                    trimmed.to_string()
                };

                let resp = process_transcript(&cmd, &ctx).await;
                let json = serde_json::to_string(&resp).unwrap_or_default();
                let _ = writer.write_all(format!("{}\n", json).as_bytes()).await;
                let _ = writer.flush().await;
            }
        }
    }
}

async fn handle_http_request(stream: &mut TcpStream, request: &str, ctx: &ServerContext) {
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        let cors_resp = "HTTP/1.1 200 OK\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Content-Length: 0\r\n\
            Connection: close\r\n\r\n";
        let _ = stream.write_all(cors_resp.as_bytes()).await;
        return;
    }

    if method == "GET" && (path == "/ping" || path == "/api/status" || path == "/") {
        let context_data = fetch_desktop_context(ctx).await;
        let response_payload = serde_json::json!({
            "status": "ok",
            "app": "reflex",
            "version": "1.1.1",
            "context": context_data
        });
        let body = serde_json::to_string(&response_payload).unwrap_or_default();
        let resp = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: application/json\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }

    if method == "GET" && path == "/api/context" {
        let context_data = fetch_desktop_context(ctx).await;
        let body = serde_json::to_string(&context_data).unwrap_or_default();
        let resp = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: application/json\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }

    if method == "POST" && (path == "/api/command" || path == "/") {
        let body = if let Some(pos) = request.find("\r\n\r\n") {
            &request[pos + 4..]
        } else if let Some(pos) = request.find("\n\n") {
            &request[pos + 2..]
        } else {
            ""
        };

        let command_str = if let Ok(payload) = serde_json::from_str::<CommandPayload>(body) {
            payload.command
        } else {
            body.trim().to_string()
        };

        let resp_obj = process_transcript(&command_str, ctx).await;
        let resp_json = serde_json::to_string(&resp_obj).unwrap_or_default();
        let resp = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: application/json\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n{}",
            resp_json.len(),
            resp_json
        );
        let _ = stream.write_all(resp.as_bytes()).await;
        return;
    }

    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let _ = stream.write_all(not_found.as_bytes()).await;
}
