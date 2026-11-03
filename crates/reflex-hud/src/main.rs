mod icons;
mod render;
mod spring;
mod wayland;

use calloop::timer::{TimeoutAction, Timer};
use calloop::EventLoop;
use icons::LucideIcon;
use render::{PillItem, PillTone};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    reexports::calloop_wayland_source::WaylandSource,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm},
};
use std::env;
use std::io::BufRead;
use std::time::Duration;
use wayland::HudState;
use wayland_client::globals::registry_queue_init;
use wayland_client::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Starting Reflex HUD (Hex-Matched Pure Rust Wayland Layer-Shell)...");

    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    let mut margin_top = 56; // Floats comfortably below top bars / Hex HUD
    let mut hold_ms = 1800;
    let mut demo_mode = false;
    let mut reduce_motion = env::var("REFLEX_REDUCED_MOTION").is_ok()
        || env::var("REDUCE_MOTION").is_ok()
        || env::var("GTK_ANIMATION_ENABLED").is_ok_and(|v| v == "0");

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--margin-top" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<i32>() {
                margin_top = val;
            }
            i += 2;
            continue;
        }
        if args[i] == "--hold" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<u64>() {
                hold_ms = val;
            }
            i += 2;
            continue;
        }
        if args[i] == "--reduce-motion" {
            reduce_motion = true;
            i += 1;
            continue;
        }
        if args[i] == "--demo" || args[i] == "--test" {
            demo_mode = true;
            i += 1;
            continue;
        }
        i += 1;
    }

    println!(
        "⚡ Reflex HUD configuration: margin_top = {}px, hold = {}ms, demo = {}, reduce_motion = {}",
        margin_top, hold_ms, demo_mode, reduce_motion
    );

    // 1. Connect to Wayland compositor (Hyprland)
    let conn = Connection::connect_to_env().inspect_err(|_| {
        eprintln!("Failed to connect to Wayland display. Ensure WAYLAND_DISPLAY is set.");
    })?;

    let (globals, mut event_queue) = registry_queue_init::<HudState>(&conn)?;
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);
    let compositor_state = CompositorState::bind(&globals, &qh)?;
    let output_state = OutputState::new(&globals, &qh);
    let shm = Shm::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let pool = SlotPool::new(480 * 64 * 4, &shm)?;

    let width = 480;
    let height = 64;

    // Test Demo Items matching Reflex voice / automation commands with Lucide Vector SVGs (no ms stats)
    let demo_items = vec![
        PillItem {
            icon: LucideIcon::VolumeX,
            title: "Muted Brave".to_string(),
            detail: Some("active stream".to_string()),
            tone: PillTone::Emerald,
        },
        PillItem {
            icon: LucideIcon::Layers,
            title: "Workspace 3".to_string(),
            detail: None,
            tone: PillTone::Cyan,
        },
        PillItem {
            icon: LucideIcon::Move,
            title: "Focus Left".to_string(),
            detail: Some("nvim".to_string()),
            tone: PillTone::Cyan,
        },
        PillItem {
            icon: LucideIcon::ShieldAlert,
            title: "Volume Clamped".to_string(),
            detail: Some("velocity limit".to_string()),
            tone: PillTone::Amber,
        },
        PillItem {
            icon: LucideIcon::Lock,
            title: "Close Kitty?".to_string(),
            detail: Some("Say 'confirm'".to_string()),
            tone: PillTone::Coral,
        },
        PillItem {
            icon: LucideIcon::Camera,
            title: "Window Snip".to_string(),
            detail: Some("to clipboard".to_string()),
            tone: PillTone::Emerald,
        },
        PillItem {
            icon: LucideIcon::Headphones,
            title: "Audio Swapped".to_string(),
            detail: Some("Headphones".to_string()),
            tone: PillTone::Emerald,
        },
        PillItem {
            icon: LucideIcon::Sparkles,
            title: "Reflex AI Dispatch".to_string(),
            detail: Some("Antigravity".to_string()),
            tone: PillTone::Purple,
        },
    ];

    let mut hud_state = HudState::new(
        registry_state,
        output_state,
        shm,
        pool,
        width,
        height,
        qh.clone(),
        demo_items,
        Duration::from_millis(hold_ms),
        reduce_motion,
        demo_mode,
    );

    // 2. Create the LayerSurface on Layer::Overlay
    let surface = compositor_state.create_surface(&qh);

    // Set empty input region -> 100% click-through (Hex pattern)
    let empty_region = compositor_state.wl_compositor().create_region(&qh, ());
    surface.set_input_region(Some(&empty_region));

    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay,
        Some("reflex-hud".to_string()),
        None, // Active output
    );

    layer.set_anchor(Anchor::TOP);
    layer.set_margin(margin_top, 0, 0, 0);
    layer.set_size(width, height);
    layer.set_exclusive_zone(0);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    hud_state.layer = Some(layer);

    // Initial roundtrip to configure surface
    event_queue.roundtrip(&mut hud_state)?;

    // 3. Set up calloop event loop (Wayland events + 60 FPS animation timer + IPC channel)
    let mut event_loop: EventLoop<HudState> = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();

    // Hook Wayland socket events into calloop
    WaylandSource::new(conn, event_queue).insert(loop_handle.clone())?;

    // Hook IPC events from reflexd into calloop
    let (ipc_tx, ipc_rx) = calloop::channel::channel::<reflex_core::hud::HudEvent>();
    loop_handle.insert_source(ipc_rx, |event, _, state: &mut HudState| {
        if let calloop::channel::Event::Msg(hud_event) = event {
            state.show_event(hud_event);
        }
    })?;

    if !demo_mode {
        let socket_path = reflex_core::hud::get_hud_socket_path();
        std::thread::Builder::new()
            .name("reflex-hud-ipc".to_string())
            .spawn(move || {
                let mut retry_delay = Duration::from_millis(200);
                loop {
                    match std::os::unix::net::UnixStream::connect(&socket_path) {
                        Ok(stream) => {
                            retry_delay = Duration::from_millis(200);
                            let reader = std::io::BufReader::new(stream);
                            for line in reader.lines() {
                                match line {
                                    Ok(text) => {
                                        let trimmed = text.trim();
                                        if trimmed.is_empty() {
                                            continue;
                                        }
                                        if let Ok(event) = serde_json::from_str::<reflex_core::hud::HudEvent>(trimmed) {
                                            if ipc_tx.send(event).is_err() {
                                                return; // Main event loop exited
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                        Err(_) => {
                            std::thread::sleep(retry_delay);
                            if retry_delay < Duration::from_secs(2) {
                                retry_delay += Duration::from_millis(300);
                            }
                        }
                    }
                }
            })?;
    }

    // Timer firing every ~16ms for smooth 60 FPS spring animation
    let timer = Timer::immediate();
    loop_handle.insert_source(timer, move |_deadline, _, state: &mut HudState| {
        state.tick_animation();
        TimeoutAction::ToDuration(Duration::from_millis(16))
    })?;

    if demo_mode {
        println!("⚡ Reflex HUD running in DEMO mode! Cycling test events...");
    } else {
        println!("⚡ Reflex HUD running in LISTENER mode! Connected to broadcast stream at {}", reflex_core::hud::get_hud_socket_path().display());
    }

    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut hud_state)?;
    }
}
