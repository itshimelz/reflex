//! HUD event models and IPC definitions for Reflex.

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudTone {
    Emerald, // Success / Deterministic action
    Cyan,    // Navigation / Workspace
    Amber,   // Warning / Guardrail limit
    Coral,   // Confirmation / Protected target
    Purple,  // AI Fallback
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudIcon {
    Volume2,
    VolumeX,
    Volume1,
    Layers,
    Move,
    Pin,
    Maximize2,
    ShieldAlert,
    Lock,
    Clipboard,
    Camera,
    Headphones,
    Speaker,
    Zap,
    Check,
    AlertTriangle,
    Sparkles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudEvent {
    pub icon: HudIcon,
    pub title: String,
    pub detail: Option<String>,
    pub tone: HudTone,
    pub hold_ms: Option<u64>,
}

impl HudEvent {
    pub fn new(icon: HudIcon, title: impl Into<String>, tone: HudTone) -> Self {
        Self {
            icon,
            title: title.into(),
            detail: None,
            tone,
            hold_ms: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_hold(mut self, hold_ms: u64) -> Self {
        self.hold_ms = Some(hold_ms);
        self
    }
}

pub fn get_hud_socket_path() -> PathBuf {
    let xdg = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    PathBuf::from(xdg).join("reflex-hud.sock")
}
