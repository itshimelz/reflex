//! Local LAN network listener for remote mobile/desktop automation.
//!
//! Listens on `0.0.0.0:9421` and accepts:
//! 1. HTTP REST requests:
//!    - `GET /ping` or `GET /api/status` -> 200 OK `{"status":"ok","app":"reflex","version":"1.1.1"}`
//!    - `POST /api/command` -> executes command, returns JSON `IpcResponse`
//!    - `OPTIONS *` -> CORS headers
//! 2. Raw TCP streams -> reads newline-delimited command, replies with JSON line.

use crate::server::{process_transcript, ServerContext};
use serde::Deserialize;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

pub const DEFAULT_NETWORK_PORT: u16 = 9421;

#[derive(Deserialize)]
struct CommandPayload {
    command: String,
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
    let mut buffer = [0u8; 8192];
    let bytes_read = match stream.read(&mut buffer).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let raw_data = String::from_utf8_lossy(&buffer[..bytes_read]);

    // Check if it's an HTTP request
    if raw_data.starts_with("GET ")
        || raw_data.starts_with("POST ")
        || raw_data.starts_with("OPTIONS ")
    {
        handle_http_request(&mut stream, &raw_data, &ctx).await;
    } else {
        // Raw TCP line
        let first_line = raw_data.lines().next().unwrap_or("").trim();
        let resp = process_transcript(first_line, &ctx).await;
        let json = serde_json::to_string(&resp).unwrap_or_default();
        let _ = stream.write_all(format!("{}\n", json).as_bytes()).await;
        let _ = stream.flush().await;
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
        let body = r#"{"status":"ok","app":"reflex","version":"1.1.1"}"#;
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
        // Extract body
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
