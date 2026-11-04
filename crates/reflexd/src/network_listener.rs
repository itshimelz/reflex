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

#[derive(Clone)]
struct CachedArt {
    bytes: Vec<u8>,
    content_type: String,
    track_id: String,
}

static CURRENT_CACHED_ART: std::sync::RwLock<Option<CachedArt>> = std::sync::RwLock::new(None);

#[derive(Deserialize)]
struct CommandPayload {
    command: String,
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
    pub position_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub art_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub percent: u8,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopWindowPayload {
    pub address: String,
    pub class: String,
    pub title: String,
    pub app_name: String,
    pub workspace: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePayload {
    pub id: i32,
    pub name: String,
    pub is_active: bool,
    pub window_count: usize,
    pub windows: Vec<DesktopWindowPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopContextPayload {
    pub active_window: Option<WindowInfo>,
    pub media: Option<MediaInfo>,
    pub volume: Option<VolumeInfo>,
    #[serde(default)]
    pub active_workspace: Option<i32>,
    #[serde(default)]
    pub workspaces: Vec<WorkspacePayload>,
    #[serde(default)]
    pub windows: Vec<DesktopWindowPayload>,
}

#[derive(Deserialize)]
struct RawActiveWs {
    id: i32,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct RawWsItem {
    id: i32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    windows: usize,
}

#[derive(Deserialize)]
struct RawClientItem {
    #[serde(default)]
    address: String,
    #[serde(default)]
    class: String,
    #[serde(default)]
    title: String,
    workspace: RawClientWs,
    #[serde(default, rename = "focusHistoryID")]
    focus_history_id: i32,
}

#[derive(Deserialize)]
struct RawClientWs {
    id: i32,
    #[serde(default)]
    name: String,
}

pub fn resolve_app_display_name(class: &str, title: &str) -> String {
    let class_lower = class.to_lowercase();
    let title_lower = title.to_lowercase();

    if class_lower.contains("dolphin") || class_lower.contains("nautilus") || class_lower.contains("thunar") {
        "Dolphin".to_string()
    } else if class_lower.contains("brave") {
        if title_lower.contains("youtube") {
            "YouTube".to_string()
        } else {
            "Brave".to_string()
        }
    } else if class_lower.contains("chrome") {
        if title_lower.contains("youtube") {
            "YouTube".to_string()
        } else {
            "Chrome".to_string()
        }
    } else if class_lower.contains("firefox") {
        if title_lower.contains("youtube") {
            "YouTube".to_string()
        } else {
            "Firefox".to_string()
        }
    } else if class_lower.contains("spotify") {
        "Spotify".to_string()
    } else if class_lower.contains("kitty") {
        "Kitty".to_string()
    } else if class_lower.contains("alacritty") {
        "Alacritty".to_string()
    } else if class_lower.contains("ghostty") {
        "Ghostty".to_string()
    } else if class_lower.contains("terminal") || class_lower.contains("foot") {
        "Terminal".to_string()
    } else if class_lower.contains("antigravity") {
        "Antigravity".to_string()
    } else if class_lower.contains("code") || class_lower.contains("cursor") {
        "VS Code".to_string()
    } else if class_lower.contains("nvim") || class_lower.contains("neovim") {
        "Neovim".to_string()
    } else if class_lower.contains("discord") || class_lower.contains("vesktop") {
        "Discord".to_string()
    } else if class_lower.contains("slack") {
        "Slack".to_string()
    } else if class_lower.contains("telegram") {
        "Telegram".to_string()
    } else if !class.is_empty() {
        let clean = class.split('-').next().unwrap_or(class);
        let mut chars = clean.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        "Application".to_string()
    }
}

pub async fn fetch_desktop_context(ctx: &ServerContext) -> DesktopContextPayload {
    // 1. Fetch Hyprland active window, workspaces, and open client windows
    let (active_window, active_workspace, workspaces, windows) = if let Some(ref hl) = ctx.hyprland {
        let active_win = match hl.fetch_active_window().await {
            Ok(Some(w)) => {
                let app_name = resolve_app_display_name(&w.class, &w.title).to_lowercase();
                Some((
                    WindowInfo {
                        class: w.class.clone(),
                        title: w.title.clone(),
                        app_name,
                        workspace: w.workspace,
                    },
                    w.address.clone(),
                ))
            }
            _ => None,
        };

        let active_addr = active_win.as_ref().map(|(_, addr)| addr.clone()).unwrap_or_default();
        let win_info = active_win.map(|(w, _)| w);

        let active_ws_id = match hl.dispatch_raw("j/activeworkspace").await {
            Ok(json_str) => serde_json::from_str::<RawActiveWs>(&json_str).ok().map(|a| a.id),
            _ => None,
        }.or_else(|| win_info.as_ref().map(|w| w.workspace as i32));

        let raw_workspaces: Vec<RawWsItem> = match hl.dispatch_raw("j/workspaces").await {
            Ok(json_str) => serde_json::from_str::<Vec<RawWsItem>>(&json_str).unwrap_or_default(),
            _ => Vec::new(),
        };

        let raw_clients: Vec<RawClientItem> = match hl.dispatch_raw("j/clients").await {
            Ok(json_str) => serde_json::from_str::<Vec<RawClientItem>>(&json_str).unwrap_or_default(),
            _ => Vec::new(),
        };

        let mut parsed_windows: Vec<DesktopWindowPayload> = Vec::new();
        for client in raw_clients {
            if client.workspace.id <= 0 {
                continue;
            }
            let is_active = (!active_addr.is_empty() && client.address == active_addr) || client.focus_history_id == 0;
            let friendly_name = resolve_app_display_name(&client.class, &client.title);
            parsed_windows.push(DesktopWindowPayload {
                address: client.address,
                class: client.class,
                title: client.title,
                app_name: friendly_name,
                workspace: client.workspace.id,
                is_active,
            });
        }

        // Group into workspaces
        let mut ws_map: std::collections::BTreeMap<i32, (String, Vec<DesktopWindowPayload>)> = std::collections::BTreeMap::new();

        for rw in raw_workspaces {
            if rw.id > 0 {
                ws_map.entry(rw.id).or_insert_with(|| (rw.name, Vec::new()));
            }
        }

        for w in &parsed_windows {
            let entry = ws_map.entry(w.workspace).or_insert_with(|| (w.workspace.to_string(), Vec::new()));
            entry.1.push(w.clone());
        }

        let mut final_workspaces: Vec<WorkspacePayload> = Vec::new();
        for (id, (name, wins)) in ws_map {
            let is_act = active_ws_id == Some(id);
            let count = wins.len();
            final_workspaces.push(WorkspacePayload {
                id,
                name: if name.is_empty() { id.to_string() } else { name },
                is_active: is_act,
                window_count: count,
                windows: wins,
            });
        }

        (win_info, active_ws_id, final_workspaces, parsed_windows)
    } else {
        (None, None, Vec::new(), Vec::new())
    };

    // 2. Fetch media info via playerctl (querying all players with -a to find the richest metadata)
    let media = match tokio::process::Command::new("playerctl")
        .args([
            "-a",
            "metadata",
            "--format",
            "{{title}}|||{{artist}}|||{{status}}|||{{playerName}}|||{{position}}|||{{mpris:length}}|||{{mpris:artUrl}}|||{{xesam:url}}",
        ])
        .output()
        .await
    {
        Ok(out) if out.status.success() => {
            let output_str = String::from_utf8_lossy(&out.stdout);
            let mut candidates: Vec<MediaInfo> = Vec::new();

            for line in output_str.lines() {
                let s = line.trim();
                if s.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = s.split("|||").collect();
                if parts.len() >= 4 {
                    let title = parts[0].trim().to_string();
                    let artist = parts[1].trim().to_string();
                    let status = parts[2].trim().to_string();
                    let player = parts[3].trim().to_string();

                    if title.is_empty() && artist.is_empty() {
                        continue;
                    }

                    let position_seconds = parts
                        .get(4)
                        .and_then(|p| p.trim().parse::<f64>().ok())
                        .map(|p| p / 1_000_000.0);

                    let duration_seconds = parts
                        .get(5)
                        .and_then(|l| l.trim().parse::<f64>().ok())
                        .filter(|&d| d > 0.0)
                        .map(|d| d / 1_000_000.0);

                    let raw_art = parts.get(6).map(|a| a.trim().to_string()).unwrap_or_default();
                    let media_url = parts.get(7).map(|u| u.trim().to_string()).unwrap_or_default();

                    // 1. YouTube direct CDN thumbnail has highest precedence (universal & fast)
                    let yt_vid = if media_url.contains("youtube.com/watch?v=") {
                        media_url
                            .split("watch?v=")
                            .nth(1)
                            .and_then(|s| s.split('&').next())
                    } else if media_url.contains("youtu.be/") {
                        media_url
                            .split("youtu.be/")
                            .nth(1)
                            .and_then(|s| s.split('?').next())
                    } else {
                        None
                    };

                    let art_url = if let Some(vid) = yt_vid.filter(|v| !v.is_empty()) {
                        Some(format!("https://img.youtube.com/vi/{}/hqdefault.jpg", vid))
                    } else if raw_art.starts_with("http://") || raw_art.starts_with("https://") {
                        Some(raw_art)
                    } else if raw_art.starts_with("file://") {
                        let local_path = raw_art.trim_start_matches("file://").to_string();
                        let track_id = format!("{:x}", (title.len() * 31 + artist.len()));

                        // Read file bytes immediately into in-memory cache so deletion by player won't break it
                        if let Ok(bytes) = std::fs::read(&local_path) {
                            let content_type = if local_path.ends_with(".png") {
                                "image/png"
                            } else if local_path.ends_with(".webp") {
                                "image/webp"
                            } else {
                                "image/jpeg"
                            };
                            if let Ok(mut lock) = CURRENT_CACHED_ART.write() {
                                *lock = Some(CachedArt {
                                    bytes,
                                    content_type: content_type.to_string(),
                                    track_id: track_id.clone(),
                                });
                            }
                        }
                        Some(format!("/api/media/art?k={}", track_id))
                    } else {
                        None
                    };

                    candidates.push(MediaInfo {
                        title,
                        artist,
                        status,
                        player,
                        position_seconds,
                        duration_seconds,
                        art_url,
                    });
                }
            }

            // Pick best candidate: prefer Playing, then prefer one with art_url, then with artist
            candidates.sort_by(|a, b| {
                let a_score = (if a.status.eq_ignore_ascii_case("playing") { 10 } else { 0 })
                    + (if a.art_url.is_some() { 5 } else { 0 })
                    + (if !a.artist.is_empty() { 3 } else { 0 });
                let b_score = (if b.status.eq_ignore_ascii_case("playing") { 10 } else { 0 })
                    + (if b.art_url.is_some() { 5 } else { 0 })
                    + (if !b.artist.is_empty() { 3 } else { 0 });
                b_score.cmp(&a_score)
            });

            // If winning candidate lacks art_url, borrow from another candidate with art_url
            let donor_art = candidates.iter().find_map(|c| c.art_url.clone());
            if let Some(first) = candidates.first_mut() {
                if first.art_url.is_none() {
                    first.art_url = donor_art;
                }
            }

            candidates.into_iter().next()
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
        active_workspace,
        workspaces,
        windows,
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
    let clean_path = path.split('?').next().unwrap_or(path);

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

    if method == "GET" && (clean_path == "/ping" || clean_path == "/api/status" || clean_path == "/") {
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

    if method == "GET" && clean_path == "/api/context" {
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

    if method == "GET" && clean_path == "/api/history" {
        let history = ctx.command_history.read().await;
        let entries: Vec<_> = history.iter().cloned().collect();
        let body = serde_json::to_string(&entries).unwrap_or_default();
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

    if method == "GET" && clean_path == "/api/media/art" {
        let cached = CURRENT_CACHED_ART
            .read()
            .ok()
            .and_then(|guard| guard.clone());

        if let Some(art) = cached {
            let resp_header = format!(
                "HTTP/1.1 200 OK\r\n\
                Content-Type: {}\r\n\
                Access-Control-Allow-Origin: *\r\n\
                Content-Length: {}\r\n\
                Cache-Control: public, max-age=86400\r\n\
                Connection: close\r\n\r\n",
                art.content_type,
                art.bytes.len()
            );

            let _ = stream.write_all(resp_header.as_bytes()).await;
            let _ = stream.write_all(&art.bytes).await;
            let _ = stream.flush().await;
            return;
        }

        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
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
