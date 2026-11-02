//! reflexd: Resident engine daemon for Reflex.

use reflex_core::config::ReflexConfig;
use reflex_core::guardrails::GuardrailEngine;
use reflex_core::registry::AppRegistry;
use reflexd::audio::AudioDriver;
use reflexd::hyprland::HyprlandDriver;
use reflexd::notify::NotificationDriver;
use reflexd::server::{run_server, ServerContext};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    info!("Starting reflexd (Reflex Resident Automation Daemon)...");

    // 1. Load configuration
    let config = ReflexConfig::load();
    info!(
        "Loaded Reflex configuration: volume ceiling={}%, velocity={}%, timeout={}s",
        config.volume.ceiling,
        config.volume.max_step_velocity,
        config.guardrails.confirmation_timeout_secs
    );

    // 2. Initialize AppRegistry and harvest .desktop files
    let mut registry = AppRegistry::new();
    registry.set_aliases(config.aliases.clone());

    let sys_dir = PathBuf::from("/usr/share/applications");
    let user_dir = std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".local/share/applications"));

    let mut search_dirs = vec![sys_dir.as_path()];
    if let Some(ref ud) = user_dir {
        search_dirs.push(ud.as_path());
    }
    registry.harvest(&search_dirs);
    info!(
        "Indexed {} desktop entries into AppRegistry",
        registry.desktop_entries.len()
    );

    // 3. Initialize GuardrailEngine
    let mut guardrails = GuardrailEngine::default();
    guardrails.volume_ceiling = config.volume.ceiling;
    guardrails.max_step_velocity = config.volume.max_step_velocity;
    guardrails.confirmation_timeout =
        Duration::from_secs_f32(config.guardrails.confirmation_timeout_secs);
    guardrails.unkillable_classes = config
        .guardrails
        .unkillable_classes
        .iter()
        .cloned()
        .collect();

    // 4. Initialize Drivers
    let audio = Arc::new(AudioDriver::new().await?);
    let notify = Arc::new(NotificationDriver::new().await?);

    let hyprland = match HyprlandDriver::new() {
        Ok(driver) => {
            let driver_arc = Arc::new(driver);
            driver_arc.clone().start_event_listener();
            info!("Hyprland dual-socket driver initialized successfully");
            Some(driver_arc)
        }
        Err(e) => {
            warn!("Hyprland dual-socket driver not active: {}. Running in standalone audio/mock mode.", e);
            None
        }
    };

    // 5. Build Server Context
    let clipboard = Arc::new(reflexd::clipboard::ClipboardDriver::new());
    let ctx = Arc::new(ServerContext {
        hyprland,
        audio,
        notify,
        clipboard,
        registry: Arc::new(RwLock::new(registry)),
        guardrails: Arc::new(Mutex::new(guardrails)),
        config,
    });

    // 6. Run IPC Server with graceful shutdown
    tokio::select! {
        res = run_server(ctx) => {
            if let Err(e) = res {
                error!("reflexd server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received SIGINT; shutting down reflexd cleanly");
        }
    }

    Ok(())
}
