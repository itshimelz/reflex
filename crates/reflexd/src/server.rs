//! UNIX domain socket IPC server handling reflex CLI connections.

use crate::audio::AudioDriver;
use crate::hyprland::HyprlandDriver;
use crate::notify::NotificationDriver;
use crate::screenshot::ScreenshotActor;
use reflex_core::ast::*;
use reflex_core::config::ReflexConfig;
use reflex_core::earcon::EarconKind;
use reflex_core::guardrails::{GuardrailDecision, GuardrailEngine};
use reflex_core::normalizer::normalize_transcript;
use reflex_core::parser::parse_action;
use reflex_core::registry::AppRegistry;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clamped_to: Option<u8>,
    pub duration_us: u128,
}

#[allow(dead_code)]
pub struct ServerContext {
    pub hyprland: Option<Arc<HyprlandDriver>>,
    pub audio: Arc<AudioDriver>,
    pub notify: Arc<NotificationDriver>,
    pub clipboard: Arc<crate::clipboard::ClipboardDriver>,
    pub registry: Arc<RwLock<AppRegistry>>,
    pub guardrails: Arc<Mutex<GuardrailEngine>>,
    pub config: ReflexConfig,
}

use tokio::sync::RwLock;

pub async fn run_server(
    ctx: Arc<ServerContext>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket_path = get_socket_path();
    run_server_at_path(&socket_path, ctx).await
}

pub async fn run_server_at_path(
    socket_path: &std::path::Path,
    ctx: Arc<ServerContext>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if socket_path.exists() {
        let _ = fs::remove_file(socket_path);
    }

    let listener = UnixListener::bind(socket_path)?;
    // Enforce strict file permissions 0700 (owner only)
    let _ = fs::set_permissions(socket_path, fs::Permissions::from_mode(0o700));

    info!("reflexd IPC server listening on {:?}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let ctx_clone = Arc::clone(&ctx);
                tokio::spawn(async move {
                    handle_client(stream, ctx_clone).await;
                });
            }
            Err(e) => {
                warn!("Accept error on reflex.sock: {}", e);
            }
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    let xdg = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    PathBuf::from(xdg).join("reflex.sock")
}

async fn handle_client(stream: UnixStream, ctx: Arc<ServerContext>) {
    let start_time = Instant::now();
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    if buf_reader.read_line(&mut line).await.is_err() || line.trim().is_empty() {
        let resp = IpcResponse {
            status: "PassThrough".to_string(),
            reason: Some("EmptyInput".to_string()),
            action: None,
            clamped_to: None,
            duration_us: start_time.elapsed().as_micros(),
        };
        let _ = writer
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await;
        let _ = writer.flush().await;
        return;
    }

    let raw_transcript = line.trim();
    let normalized = normalize_transcript(raw_transcript);
    let parsed = parse_action(&normalized);

    let Some(action) = parsed else {
        // Unrecognized fast-path intent -> PassThrough to agy (exit code 2)
        let resp = IpcResponse {
            status: "PassThrough".to_string(),
            reason: Some("NoGrammarMatch".to_string()),
            action: None,
            clamped_to: None,
            duration_us: start_time.elapsed().as_micros(),
        };
        let _ = writer
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await;
        let _ = writer.flush().await;
        return;
    };

    let response = execute_action(&action, &ctx, start_time).await;
    let _ = writer
        .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
        .await;
    let _ = writer.flush().await;
}

async fn execute_action(action: &Action, ctx: &ServerContext, start_time: Instant) -> IpcResponse {
    match action {
        Action::Confirm => {
            let mut guardrails = ctx.guardrails.lock().await;
            if let Some(target_address) = guardrails.confirm() {
                drop(guardrails);
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.close_window_address(&target_address).await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                ctx.notify.notify(
                    "Window Closed",
                    "Protected window closed after confirmation",
                    "window-close",
                    2000,
                );
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("ConfirmClose".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            } else {
                IpcResponse {
                    status: "Error".to_string(),
                    reason: Some("NoPendingConfirmation".to_string()),
                    action: Some("Confirm".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
        }
        Action::Media(media) => {
            let _ = ctx.audio.handle_media(media).await;
            ctx.audio.play_earcon(EarconKind::Success).await;
            IpcResponse {
                status: "Success".to_string(),
                reason: None,
                action: Some(format!("{:?}", media)),
                clamped_to: None,
                duration_us: start_time.elapsed().as_micros(),
            }
        }
        Action::Audio(audio) => match audio {
            AudioAction::VolumeSet(target) => {
                let mut guardrails = ctx.guardrails.lock().await;
                let (clamped, decision) = guardrails.evaluate_volume_set(*target);
                drop(guardrails);

                let _ = ctx.audio.set_volume(clamped).await;

                match decision {
                    GuardrailDecision::VelocityClamped { requested, clamped } => {
                        ctx.audio.play_earcon(EarconKind::Warning).await;
                        ctx.notify.notify(
                            "⚠️ Volume Clamped",
                            &format!(
                                "Requested {}%, clamped to {}% (+15% velocity limit)",
                                requested, clamped
                            ),
                            "audio-volume-high",
                            3000,
                        );
                        IpcResponse {
                            status: "RejectedByPolicy".to_string(),
                            reason: Some("VolumeVelocityExceeded".to_string()),
                            action: Some("VolumeSet".to_string()),
                            clamped_to: Some(clamped),
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                    GuardrailDecision::CeilingClamped { requested, clamped } => {
                        ctx.audio.play_earcon(EarconKind::Warning).await;
                        ctx.notify.notify(
                            "⚠️ Volume Ceiling Enforced",
                            &format!(
                                "Requested {}%, clamped to {}% (80% maximum ceiling)",
                                requested, clamped
                            ),
                            "audio-volume-high",
                            3000,
                        );
                        IpcResponse {
                            status: "RejectedByPolicy".to_string(),
                            reason: Some("VolumeCeilingBreached".to_string()),
                            action: Some("VolumeSet".to_string()),
                            clamped_to: Some(clamped),
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                    _ => {
                        ctx.audio.play_earcon(EarconKind::Success).await;
                        IpcResponse {
                            status: "Success".to_string(),
                            reason: None,
                            action: Some("VolumeSet".to_string()),
                            clamped_to: Some(clamped),
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                }
            }
            AudioAction::VolumeStep(step) => {
                let mut guardrails = ctx.guardrails.lock().await;
                let (clamped_step, decision) = guardrails.evaluate_volume_step(*step);
                drop(guardrails);

                let _ = ctx.audio.step_volume(clamped_step).await;
                if let GuardrailDecision::VelocityClamped { clamped, .. } = decision {
                    ctx.audio.play_earcon(EarconKind::Warning).await;
                    IpcResponse {
                        status: "RejectedByPolicy".to_string(),
                        reason: Some("VolumeVelocityExceeded".to_string()),
                        action: Some("VolumeStep".to_string()),
                        clamped_to: Some(clamped),
                        duration_us: start_time.elapsed().as_micros(),
                    }
                } else {
                    ctx.audio.play_earcon(EarconKind::Success).await;
                    IpcResponse {
                        status: "Success".to_string(),
                        reason: None,
                        action: Some("VolumeStep".to_string()),
                        clamped_to: None,
                        duration_us: start_time.elapsed().as_micros(),
                    }
                }
            }
            AudioAction::Mute(m) => {
                let _ = ctx.audio.set_mute(*m).await;
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some(format!("Mute({})", m)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            AudioAction::MuteToggle => {
                let _ = ctx.audio.toggle_mute().await;
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("MuteToggle".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            AudioAction::MicMuteToggle => {
                let _ = ctx.audio.toggle_mic_mute().await;
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("MicMuteToggle".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            AudioAction::SwitchPort => {
                let _ = ctx.audio.switch_audio_port().await;
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("SwitchAudioPort".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            AudioAction::AppAudio { target, mode } => {
                match ctx.audio.handle_app_audio(target, mode).await {
                    Ok(count) => {
                        ctx.audio.play_earcon(EarconKind::Success).await;
                        let desc = match mode {
                            AppAudioMode::Mute(true) => format!("Muted {}", target),
                            AppAudioMode::Mute(false) => format!("Unmuted {}", target),
                            AppAudioMode::ToggleMute => format!("Toggled mute for {}", target),
                            AppAudioMode::VolumeSet(v) => format!("Set {} volume to {}%", target, v),
                            AppAudioMode::VolumeStep(s) => {
                                format!("Stepped {} volume by {}%", target, s)
                            }
                        };
                        ctx.notify
                            .notify("App Audio", &desc, "audio-volume-medium", 2000);
                        IpcResponse {
                            status: "Success".to_string(),
                            reason: None,
                            action: Some(format!("AppAudio({}, {} stream(s))", target, count)),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                    Err(reason) => {
                        ctx.audio.play_earcon(EarconKind::Warning).await;
                        ctx.notify
                            .notify("App Audio Failed", &reason, "dialog-warning", 3000);
                        IpcResponse {
                            status: "Error".to_string(),
                            reason: Some(reason),
                            action: Some(format!("AppAudio({})", target)),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                }
            }
        },
        Action::Window(win) => match win {
            WindowAction::CloseActive => {
                if let Some(ref hypr) = ctx.hyprland {
                    if let Ok(Some(active)) = hypr.fetch_active_window().await {
                        let mut guardrails = ctx.guardrails.lock().await;
                        let decision = guardrails.evaluate_window_close(&active);
                        drop(guardrails);

                        if let GuardrailDecision::RequiresConfirmation {
                            target_address,
                            window_class,
                        } = decision
                        {
                            ctx.audio.play_earcon(EarconKind::ConfirmationChime).await;
                            ctx.notify.notify(
                                "⏳ Confirm Close Window?",
                                &format!(
                                    "Class '{}' is protected. Say 'confirm' within 4.0s.",
                                    window_class
                                ),
                                "dialog-warning",
                                4000,
                            );
                            return IpcResponse {
                                status: "AwaitingConfirmation".to_string(),
                                reason: Some(format!("ProtectedWindow({})", window_class)),
                                action: Some(target_address),
                                clamped_to: None,
                                duration_us: start_time.elapsed().as_micros(),
                            };
                        }
                    }
                    let _ = hypr.close_active_window().await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("CloseActiveWindow".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::ToggleFullscreen => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.toggle_fullscreen().await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("ToggleFullscreen".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::ToggleFloating => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.toggle_floating().await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("ToggleFloating".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::TogglePin => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.toggle_pin().await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("TogglePin".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::CenterWindow => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.center_window().await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("CenterWindow".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::ToggleSplit => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.toggle_split().await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("ToggleSplit".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::FocusDirection(dir) => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.focus_direction(*dir).await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some(format!("FocusDirection({:?})", dir)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WindowAction::MoveDirection(dir) => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.move_window_direction(*dir).await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some(format!("MoveDirection({:?})", dir)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
        },
        Action::Workspace(ws_action) => match ws_action {
            WorkspaceAction::Focus(ws) => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.focus_workspace(*ws).await;

                    // Asynchronous decoupled verification on .socket2.sock
                    let mut rx = hypr.event_tx.subscribe();
                    let expected_ws = *ws;
                    let audio_clone = Arc::clone(&ctx.audio);
                    let notify_clone = Arc::clone(&ctx.notify);
                    tokio::spawn(async move {
                        let timeout = sleep(Duration::from_millis(200));
                        tokio::pin!(timeout);

                        loop {
                            tokio::select! {
                                _ = &mut timeout => {
                                    audio_clone.play_earcon(EarconKind::Error).await;
                                    notify_clone.notify("State Desync Detected", "Workspace switch timed out on event socket", "dialog-error", 3000);
                                    break;
                                }
                                Ok(event) = rx.recv() => {
                                    if event.starts_with(&format!("workspace>>{}", expected_ws))
                                        || event.starts_with(&format!("workspacev2>>{}", expected_ws))
                                    {
                                        break; // Verified!
                                    }
                                }
                            }
                        }
                    });
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some(format!("FocusWorkspace({})", ws)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            WorkspaceAction::MoveWindow { workspace, follow } => {
                if let Some(ref hypr) = ctx.hyprland {
                    let _ = hypr.move_window_to_workspace(*workspace, *follow).await;
                }
                ctx.audio.play_earcon(EarconKind::Success).await;
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some(format!("MoveWindow(ws={}, follow={})", workspace, follow)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
        },
        Action::MoveNamed {
            app,
            workspace,
            follow,
        } => {
            if let Some(ref hypr) = ctx.hyprland {
                if let Ok(clients) = hypr.fetch_clients().await {
                    let registry = ctx.registry.read().await;
                    if let Some((target_client, _tier)) = registry.resolve_client(app, &clients) {
                        let addr = target_client.address.clone();
                        let target_class = target_client.class.clone();
                        drop(registry);

                        let _ = hypr
                            .move_address_to_workspace(&addr, *workspace, *follow)
                            .await;
                        ctx.audio.play_earcon(EarconKind::Success).await;
                        ctx.notify.notify(
                            "Window Moved",
                            &format!("Moved {} to Workspace {}", app, workspace),
                            "window-new",
                            2500,
                        );

                        return IpcResponse {
                            status: "Success".to_string(),
                            reason: None,
                            action: Some(format!(
                                "MoveNamed(target={}, ws={})",
                                target_class, workspace
                            )),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        };
                    }
                }
            }
            ctx.audio.play_earcon(EarconKind::Error).await;
            IpcResponse {
                status: "Error".to_string(),
                reason: Some(format!("AppNotFound({})", app)),
                action: Some("MoveNamed".to_string()),
                clamped_to: None,
                duration_us: start_time.elapsed().as_micros(),
            }
        }
        Action::Screenshot(shot) => {
            match shot {
                ScreenshotAction::ActiveWindow => {
                    if let Some(ref hypr) = ctx.hyprland {
                        if let Ok(Some((x, y, w, h))) = hypr.fetch_active_window_geometry().await {
                            ScreenshotActor::capture_box(x, y, w, h);
                        } else {
                            ScreenshotActor::capture_fullscreen();
                        }
                    } else {
                        ScreenshotActor::capture_fullscreen();
                    }
                }
                ScreenshotAction::FullScreen => {
                    ScreenshotActor::capture_fullscreen();
                }
                ScreenshotAction::InteractiveSnip => {
                    ScreenshotActor::interactive_snip();
                }
                _ => {
                    ScreenshotActor::capture_fullscreen();
                }
            }

            ctx.audio.play_earcon(EarconKind::Shutter).await;
            ctx.notify.notify(
                "Screenshot Captured",
                "Saved to ~/Pictures/Screenshots & copied to clipboard",
                "camera-photo",
                2500,
            );
            IpcResponse {
                status: "Success".to_string(),
                reason: None,
                action: Some(format!("Screenshot({:?})", shot)),
                clamped_to: None,
                duration_us: start_time.elapsed().as_micros(),
            }
        }
        Action::Clipboard(clip_action) => match clip_action {
            ClipboardAction::TypeClipboard => {
                match ctx.clipboard.type_clipboard().await {
                    Ok(count) => {
                        ctx.audio.play_earcon(EarconKind::Success).await;
                        IpcResponse {
                            status: "Success".to_string(),
                            reason: None,
                            action: Some(format!("TypeClipboard({} chars)", count)),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                    Err(reason) => {
                        ctx.audio.play_earcon(EarconKind::Warning).await;
                        ctx.notify.notify(
                            "⚠️ Clipboard Typing Blocked",
                            &reason,
                            "dialog-warning",
                            3000,
                        );
                        IpcResponse {
                            status: "RejectedByPolicy".to_string(),
                            reason: Some(reason),
                            action: Some("TypeClipboard".to_string()),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        }
                    }
                }
            }
            ClipboardAction::ClearClipboard => {
                let _ = ctx.clipboard.clear().await;
                ctx.audio.play_earcon(EarconKind::Success).await;
                ctx.notify.notify("Clipboard Cleared", "Buffer emptied", "edit-clear", 2000);
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("ClearClipboard".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
            ClipboardAction::StripFormatting => {
                let _ = ctx.clipboard.strip_formatting().await;
                ctx.audio.play_earcon(EarconKind::Success).await;
                ctx.notify.notify(
                    "Clipboard Stripped",
                    "Converted to plain text",
                    "edit-paste",
                    2000,
                );
                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some("StripFormatting".to_string()),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
        },
        Action::Open(app) => {
            if let Some(ref hypr) = ctx.hyprland {
                // Tier A: Single-Instance Focus Check
                if let Ok(clients) = hypr.fetch_clients().await {
                    let registry = ctx.registry.read().await;
                    if let Some((target_client, _tier)) = registry.resolve_client(app, &clients) {
                        let addr = target_client.address.clone();
                        let target_class = target_client.class.clone();
                        let target_ws = target_client.workspace;
                        drop(registry);

                        // If window is on another workspace, switch workspace first
                        if target_ws > 0 {
                            let _ = hypr.focus_workspace(target_ws).await;
                        }
                        let _ = hypr.focus_window_address(&addr).await;

                        ctx.audio.play_earcon(EarconKind::Success).await;
                        ctx.notify.notify(
                            "Application Focused",
                            &format!("Switched focus to {}", app),
                            "window-new",
                            2500,
                        );

                        return IpcResponse {
                            status: "Success".to_string(),
                            reason: None,
                            action: Some(format!("FocusExisting({})", target_class)),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        };
                    }
                }

                // Tier B: Launch new instance via harvested .desktop Exec or fallback to app name
                let registry = ctx.registry.read().await;
                let launch_cmd = registry
                    .resolve_launch_exec(app)
                    .unwrap_or_else(|| app.to_string());
                drop(registry);

                let escaped = launch_cmd.replace('\\', "\\\\").replace('"', "\\\"");
                let _ = hypr
                    .dispatch_raw(&format!("dispatch hl.dsp.exec_cmd(\"{}\")", escaped))
                    .await;

                ctx.audio.play_earcon(EarconKind::Success).await;
                ctx.notify.notify(
                    "Application Launched",
                    &format!("Launching {}", app),
                    "system-run",
                    2500,
                );

                IpcResponse {
                    status: "Success".to_string(),
                    reason: None,
                    action: Some(format!("Launch({})", launch_cmd)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            } else {
                ctx.audio.play_earcon(EarconKind::Error).await;
                IpcResponse {
                    status: "Error".to_string(),
                    reason: Some("CompositorNotConnected".to_string()),
                    action: Some(format!("Open({})", app)),
                    clamped_to: None,
                    duration_us: start_time.elapsed().as_micros(),
                }
            }
        }
        Action::Close(app) => {
            if let Some(ref hypr) = ctx.hyprland {
                if let Ok(clients) = hypr.fetch_clients().await {
                    let registry = ctx.registry.read().await;
                    if let Some((target_client, _)) = registry.resolve_client(app, &clients) {
                        let addr = target_client.address.clone();
                        let target_class = target_client.class.clone();
                        let client_clone = target_client.clone();
                        drop(registry);

                        // Safety check: verify against guardrails unkillable classes
                        let mut guardrails = ctx.guardrails.lock().await;
                        let decision = guardrails.evaluate_window_close(&client_clone);
                        drop(guardrails);

                        if let GuardrailDecision::RequiresConfirmation {
                            target_address,
                            window_class,
                        } = decision
                        {
                            ctx.audio.play_earcon(EarconKind::ConfirmationChime).await;
                            ctx.notify.notify(
                                "⏳ Confirm Close Window?",
                                &format!(
                                    "Class '{}' is protected. Say 'confirm' within 4.0s.",
                                    window_class
                                ),
                                "dialog-warning",
                                4000,
                            );
                            return IpcResponse {
                                status: "AwaitingConfirmation".to_string(),
                                reason: Some(format!("ProtectedWindow({})", window_class)),
                                action: Some(target_address),
                                clamped_to: None,
                                duration_us: start_time.elapsed().as_micros(),
                            };
                        }

                        let _ = hypr.close_window_address(&addr).await;
                        ctx.audio.play_earcon(EarconKind::Success).await;
                        ctx.notify.notify(
                            "Window Closed",
                            &format!("Closed {}", target_class),
                            "window-close",
                            2500,
                        );
                        return IpcResponse {
                            status: "Success".to_string(),
                            reason: None,
                            action: Some(format!("Close({})", target_class)),
                            clamped_to: None,
                            duration_us: start_time.elapsed().as_micros(),
                        };
                    }
                }
            }
            ctx.audio.play_earcon(EarconKind::Error).await;
            IpcResponse {
                status: "Error".to_string(),
                reason: Some(format!("AppNotFound({})", app)),
                action: Some("Close".to_string()),
                clamped_to: None,
                duration_us: start_time.elapsed().as_micros(),
            }
        }
    }
}
