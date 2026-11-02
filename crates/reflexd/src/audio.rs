//! Audio driver managing WirePlumber volume, MPRIS playback via zbus, and rodio earcon sink.

use reflex_core::ast::MediaAction;
use reflex_core::earcon::{EarconKind, SAMPLE_RATE};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use std::thread;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use zbus::Connection;

pub struct AudioDriver {
    dbus: Connection,
    earcon_tx: mpsc::UnboundedSender<EarconKind>,
}

impl AudioDriver {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dbus = Connection::session().await?;
        let (earcon_tx, mut earcon_rx) = mpsc::unbounded_channel::<EarconKind>();

        // Dedicated audio playback thread keeping rodio::OutputStream thread-local
        thread::spawn(move || {
            let (stream, sink) = match OutputStream::try_default() {
                Ok((s, h)) => match Sink::try_new(&h) {
                    Ok(sink) => (Some(s), Some(sink)),
                    Err(e) => {
                        warn!("Failed to create rodio Sink: {}", e);
                        (Some(s), None)
                    }
                },
                Err(e) => {
                    warn!(
                        "No default audio output device available for rodio earcons: {}",
                        e
                    );
                    (None, None)
                }
            };

            let _stream = stream;

            while let Some(kind) = earcon_rx.blocking_recv() {
                if let Some(ref s) = sink {
                    let samples = kind.pcm_samples();
                    let buffer = SamplesBuffer::new(1, SAMPLE_RATE, samples);
                    s.append(buffer);
                    s.play();
                }
            }
        });

        Ok(Self { dbus, earcon_tx })
    }

    /// Play an embedded low-latency PCM earcon waveform (non-blocking)
    pub async fn play_earcon(&self, kind: EarconKind) {
        let _ = self.earcon_tx.send(kind);
    }

    /// Dispatch media action to MPRIS players on the session bus
    pub async fn handle_media(
        &self,
        action: &MediaAction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let proxy = zbus::fdo::DBusProxy::new(&self.dbus).await?;
        let names = proxy.list_names().await?;

        let mpris_players: Vec<String> = names
            .into_iter()
            .filter(|n| n.starts_with("org.mpris.MediaPlayer2."))
            .map(|n| n.to_string())
            .collect();

        if mpris_players.is_empty() {
            debug!("No active MPRIS media players detected");
            return Ok(());
        }

        for player in mpris_players {
            let dest = player.as_str();
            let path = "/org/mpris/MediaPlayer2";
            let interface = "org.mpris.MediaPlayer2.Player";

            match action {
                MediaAction::Play => {
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "Play", &())
                        .await;
                }
                MediaAction::Pause => {
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "Pause", &())
                        .await;
                }
                MediaAction::Toggle => {
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "PlayPause", &())
                        .await;
                }
                MediaAction::Next => {
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "Next", &())
                        .await;
                }
                MediaAction::Previous => {
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "Previous", &())
                        .await;
                }
                MediaAction::Replay => {
                    let track_id = zbus::zvariant::ObjectPath::from_static_str("/").unwrap();
                    let _ = self
                        .dbus
                        .call_method(
                            Some(dest),
                            path,
                            Some(interface),
                            "SetPosition",
                            &(track_id, 0i64),
                        )
                        .await;
                }
                MediaAction::SeekForward(secs) => {
                    let offset_us = (*secs as i64) * 1_000_000;
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "Seek", &(offset_us))
                        .await;
                }
                MediaAction::SeekBackward(secs) => {
                    let offset_us = -((*secs as i64) * 1_000_000);
                    let _ = self
                        .dbus
                        .call_method(Some(dest), path, Some(interface), "Seek", &(offset_us))
                        .await;
                }
            }
        }

        Ok(())
    }

    /// Set volume percentage on default audio sink (0..=100)
    pub async fn set_volume(
        &self,
        target_percent: u8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let val_f = target_percent as f64 / 100.0;
        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new("wpctl");
            cmd.args([
                "set-volume",
                "@DEFAULT_AUDIO_SINK@",
                &format!("{:.2}", val_f),
            ]);
            let _ = cmd.output().await;
        });
        Ok(())
    }

    /// Step volume percentage on default audio sink
    pub async fn step_volume(
        &self,
        step_percent: i8,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let abs_step = step_percent.abs() as f64 / 100.0;
        let arg = if step_percent >= 0 {
            format!("{:.2}+", abs_step)
        } else {
            format!("{:.2}-", abs_step)
        };
        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new("wpctl");
            cmd.args(["set-volume", "@DEFAULT_AUDIO_SINK@", &arg]);
            let _ = cmd.output().await;
        });
        Ok(())
    }

    /// Toggle mute on default audio sink
    pub async fn toggle_mute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new("wpctl");
            cmd.args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]);
            let _ = cmd.output().await;
        });
        Ok(())
    }

    /// Set mute on default audio sink
    pub async fn set_mute(
        &self,
        mute: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let arg = if mute { "1" } else { "0" };
        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new("wpctl");
            cmd.args(["set-mute", "@DEFAULT_AUDIO_SINK@", arg]);
            let _ = cmd.output().await;
        });
        Ok(())
    }

    /// Toggle mic mute on default audio source
    pub async fn toggle_mic_mute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new("wpctl");
            cmd.args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]);
            let _ = cmd.output().await;
        });
        Ok(())
    }

    /// Switch physical audio output port (Headphones <-> Speakers)
    pub async fn switch_audio_port(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tokio::spawn(async move {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/itshimelz".to_string());
            let script =
                std::path::PathBuf::from(home).join(".config/hypr/scripts/toggle_audio_port.py");
            if script.exists() {
                let mut cmd = tokio::process::Command::new("python3");
                cmd.arg(script);
                let _ = cmd.output().await;
            }
        });
        Ok(())
    }

    /// Discover PipeWire node IDs for an application target
    pub async fn find_app_sink_nodes(&self, target: &str) -> Vec<u32> {
        let clean_target = target.trim().to_lowercase();
        if clean_target.is_empty() {
            return Vec::new();
        }

        let output = match tokio::process::Command::new("pw-dump")
            .arg("Node")
            .output()
            .await
        {
            Ok(out) => out,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        let nodes: Vec<PwDumpNode> = match serde_json::from_slice(&output.stdout) {
            Ok(n) => n,
            Err(_) => return Vec::new(),
        };

        let mut matched_ids = Vec::new();
        for node in nodes {
            if let Some(info) = node.info {
                if let Some(props) = info.props {
                    if props.media_class.as_deref() == Some("Stream/Output/Audio") {
                        let matches = [
                            props.app_name.as_deref(),
                            props.app_id.as_deref(),
                            props.binary.as_deref(),
                            props.node_name.as_deref(),
                            props.media_name.as_deref(),
                        ]
                        .into_iter()
                        .flatten()
                        .any(|val| val.to_lowercase().contains(&clean_target));

                        if matches {
                            matched_ids.push(node.id);
                        }
                    }
                }
            }
        }

        matched_ids
    }

    /// Control audio stream of a specific target app
    pub async fn handle_app_audio(
        &self,
        target: &str,
        mode: &reflex_core::ast::AppAudioMode,
    ) -> Result<usize, String> {
        let node_ids = self.find_app_sink_nodes(target).await;
        if node_ids.is_empty() {
            return Err(format!("No active audio stream found for '{}'", target));
        }

        for &id in &node_ids {
            let id_str = id.to_string();
            let mut cmd = tokio::process::Command::new("wpctl");
            match mode {
                reflex_core::ast::AppAudioMode::Mute(m) => {
                    let arg = if *m { "1" } else { "0" };
                    cmd.args(["set-mute", &id_str, arg]);
                }
                reflex_core::ast::AppAudioMode::ToggleMute => {
                    cmd.args(["set-mute", &id_str, "toggle"]);
                }
                reflex_core::ast::AppAudioMode::VolumeSet(vol) => {
                    let val_f = *vol as f64 / 100.0;
                    cmd.args(["set-volume", &id_str, &format!("{:.2}", val_f)]);
                }
                reflex_core::ast::AppAudioMode::VolumeStep(step) => {
                    let abs_step = step.abs() as f64 / 100.0;
                    let arg = if *step >= 0 {
                        format!("{:.2}+", abs_step)
                    } else {
                        format!("{:.2}-", abs_step)
                    };
                    cmd.args(["set-volume", &id_str, &arg]);
                }
            }
            let _ = cmd.output().await;
        }

        Ok(node_ids.len())
    }
}

#[derive(serde::Deserialize)]
struct PwDumpNode {
    id: u32,
    #[serde(default)]
    info: Option<PwDumpNodeInfo>,
}

#[derive(serde::Deserialize)]
struct PwDumpNodeInfo {
    #[serde(default)]
    props: Option<PwDumpProps>,
}

#[derive(serde::Deserialize)]
struct PwDumpProps {
    #[serde(rename = "media.class")]
    media_class: Option<String>,
    #[serde(rename = "application.name")]
    app_name: Option<String>,
    #[serde(rename = "application.id")]
    app_id: Option<String>,
    #[serde(rename = "application.process.binary")]
    binary: Option<String>,
    #[serde(rename = "node.name")]
    node_name: Option<String>,
    #[serde(rename = "media.name")]
    media_name: Option<String>,
}
