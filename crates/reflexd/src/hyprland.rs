//! Native Hyprland dual-socket driver for reflexd.
//! Connects to .socket.sock (dispatch) and .socket2.sock (event broadcast).

use reflex_core::ast::Direction;
use reflex_core::registry::HyprlandClient;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawHyprlandWorkspace {
    pub id: i32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawHyprlandClient {
    pub address: String,
    #[serde(default)]
    pub class: String,
    #[serde(default)]
    pub title: String,
    pub workspace: RawHyprlandWorkspace,
    #[serde(default)]
    pub at: (i32, i32),
    #[serde(default)]
    pub size: (i32, i32),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HyprlandState {
    pub active_workspace: u8,
    pub active_window: Option<HyprlandClient>,
    pub clients: Vec<HyprlandClient>,
}

pub struct HyprlandDriver {
    cmd_socket_path: PathBuf,
    evt_socket_path: PathBuf,
    pub state: Arc<RwLock<HyprlandState>>,
    pub event_tx: broadcast::Sender<String>,
}

impl HyprlandDriver {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let xdg_runtime =
            env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        let his = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set; is Hyprland running?")?;

        let base = PathBuf::from(xdg_runtime).join("hypr").join(his);
        let cmd_socket_path = base.join(".socket.sock");
        let evt_socket_path = base.join(".socket2.sock");

        let (event_tx, _) = broadcast::channel(64);

        Ok(Self {
            cmd_socket_path,
            evt_socket_path,
            state: Arc::new(RwLock::new(HyprlandState {
                active_workspace: 1,
                active_window: None,
                clients: Vec::new(),
            })),
            event_tx,
        })
    }

    /// Dispatch command string to Hyprland's .socket.sock and read the response
    pub async fn dispatch_raw(
        &self,
        cmd: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = UnixStream::connect(&self.cmd_socket_path).await?;
        stream.write_all(cmd.as_bytes()).await?;
        stream.shutdown().await?;

        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response.trim().to_string())
    }

    /// Dispatch standard compositor command
    pub async fn dispatch(
        &self,
        cmd: &str,
        args: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let full = if args.is_empty() {
            format!("dispatch {}", cmd)
        } else {
            format!("dispatch {} {}", cmd, args)
        };
        self.dispatch_raw(&full).await
    }

    /// Query open clients via j/clients
    pub async fn fetch_clients(
        &self,
    ) -> Result<Vec<HyprlandClient>, Box<dyn std::error::Error + Send + Sync>> {
        let res = self.dispatch_raw("j/clients").await?;
        let raw: Vec<RawHyprlandClient> = serde_json::from_str(&res)?;
        let clients = raw
            .into_iter()
            .map(|c| HyprlandClient {
                address: c.address,
                class: c.class,
                title: c.title,
                workspace: if c.workspace.id > 0 {
                    c.workspace.id as u8
                } else {
                    1
                },
            })
            .collect();
        Ok(clients)
    }

    /// Query active window via j/activewindow
    pub async fn fetch_active_window(
        &self,
    ) -> Result<Option<HyprlandClient>, Box<dyn std::error::Error + Send + Sync>> {
        let res = self.dispatch_raw("j/activewindow").await?;
        if res.is_empty() || res == "{}" {
            return Ok(None);
        }
        let raw: RawHyprlandClient = serde_json::from_str(&res)?;
        Ok(Some(HyprlandClient {
            address: raw.address,
            class: raw.class,
            title: raw.title,
            workspace: if raw.workspace.id > 0 {
                raw.workspace.id as u8
            } else {
                1
            },
        }))
    }

    /// Query active window geometry (x, y, w, h)
    pub async fn fetch_active_window_geometry(
        &self,
    ) -> Result<Option<(i32, i32, i32, i32)>, Box<dyn std::error::Error + Send + Sync>> {
        let res = self.dispatch_raw("j/activewindow").await?;
        if res.is_empty() || res == "{}" {
            return Ok(None);
        }
        let raw: RawHyprlandClient = serde_json::from_str(&res)?;
        Ok(Some((raw.at.0, raw.at.1, raw.size.0, raw.size.1)))
    }

    /// Focus a workspace
    pub async fn focus_workspace(
        &self,
        ws: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw(&format!("dispatch hl.dsp.focus({{ workspace = {} }})", ws))
            .await?;
        Ok(())
    }

    /// Move active window to workspace
    pub async fn move_window_to_workspace(
        &self,
        ws: u8,
        follow: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw(&format!(
            "dispatch hl.dsp.window.move({{ workspace = {}, follow = {} }})",
            ws, follow
        ))
        .await?;
        Ok(())
    }

    /// Move specific window address to workspace
    pub async fn move_address_to_workspace(
        &self,
        address: &str,
        ws: u8,
        follow: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw(&format!(
            "dispatch hl.dsp.window.move({{ workspace = {}, address = \"{}\", follow = {} }})",
            ws, address, follow
        ))
        .await?;
        Ok(())
    }

    /// Close active window
    pub async fn close_active_window(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw("dispatch hl.dsp.window.close()").await?;
        Ok(())
    }

    /// Close specific window address
    pub async fn close_window_address(
        &self,
        address: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw(&format!(
            "dispatch hl.dsp.window.close({{ address = \"{}\" }})",
            address
        ))
        .await?;
        Ok(())
    }

    /// Focus specific window address
    pub async fn focus_window_address(
        &self,
        address: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw(&format!(
            "dispatch hl.dsp.focus({{ window = \"address:{}\" }})",
            address
        ))
        .await?;
        Ok(())
    }

    /// Toggle fullscreen
    pub async fn toggle_fullscreen(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw(
            "dispatch hl.dsp.window.fullscreen({ mode = \"fullscreen\", action = \"toggle\" })",
        )
        .await?;
        Ok(())
    }

    /// Toggle floating mode for active window
    pub async fn toggle_floating(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw("dispatch hl.dsp.window.float({ action = \"toggle\" })")
            .await?;
        Ok(())
    }

    /// Toggle pin for active window (pins floating window across workspaces)
    pub async fn toggle_pin(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw("dispatch hl.dsp.window.pin()").await?;
        Ok(())
    }

    /// Center active floating window
    pub async fn center_window(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw("dispatch hl.dsp.window.center()").await?;
        Ok(())
    }

    /// Toggle dwindle split orientation
    pub async fn toggle_split(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dispatch_raw("dispatch hl.dsp.layout(\"togglesplit\")").await?;
        Ok(())
    }

    /// Move focus in direction
    pub async fn focus_direction(
        &self,
        dir: Direction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let arg = match dir {
            Direction::Left => "l",
            Direction::Right => "r",
            Direction::Up => "u",
            Direction::Down => "d",
        };
        self.dispatch_raw(&format!(
            "dispatch hl.dsp.focus({{ direction = \"{}\" }})",
            arg
        ))
        .await?;
        Ok(())
    }

    /// Move window in direction
    pub async fn move_window_direction(
        &self,
        dir: Direction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let arg = match dir {
            Direction::Left => "l",
            Direction::Right => "r",
            Direction::Up => "u",
            Direction::Down => "d",
        };
        self.dispatch_raw(&format!(
            "dispatch hl.dsp.window.move({{ direction = \"{}\" }})",
            arg
        ))
        .await?;
        Ok(())
    }

    /// Start background event listener task on .socket2.sock
    pub fn start_event_listener(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                match UnixStream::connect(&self.evt_socket_path).await {
                    Ok(stream) => {
                        info!(
                            "Connected to Hyprland event socket: {:?}",
                            self.evt_socket_path
                        );
                        let reader = BufReader::new(stream);
                        let mut lines = reader.lines();

                        while let Ok(Some(line)) = lines.next_line().await {
                            let _ = self.event_tx.send(line.clone());

                            // Parse high-frequency events
                            if let Some(rest) = line.strip_prefix("workspace>>") {
                                if let Ok(ws) = rest.parse::<u8>() {
                                    let mut state = self.state.write().await;
                                    state.active_workspace = ws;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Hyprland event socket disconnected ({}); retrying in 500ms...",
                            e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        });
    }
}
