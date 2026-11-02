//! Configuration schema and loader for Reflex (~/.config/reflex/config.toml).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn default_ceiling() -> u8 {
    80
}

fn default_velocity() -> u8 {
    15
}

fn default_unkillable() -> Vec<String> {
    vec![
        "kitty".to_string(),
        "neovim".to_string(),
        "nvim".to_string(),
        "brave".to_string(),
        "brave-browser".to_string(),
        "code".to_string(),
        "vscode".to_string(),
        "antigravity".to_string(),
    ]
}

fn default_timeout_secs() -> f32 {
    4.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    #[serde(default = "default_ceiling")]
    pub ceiling: u8,
    #[serde(default = "default_velocity")]
    pub max_step_velocity: u8,
}

impl Default for VolumeConfig {
    fn default() -> Self {
        Self {
            ceiling: default_ceiling(),
            max_step_velocity: default_velocity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    #[serde(default = "default_unkillable")]
    pub unkillable_classes: Vec<String>,
    #[serde(default = "default_timeout_secs")]
    pub confirmation_timeout_secs: f32,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            unkillable_classes: default_unkillable(),
            confirmation_timeout_secs: default_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReflexConfig {
    #[serde(default)]
    pub volume: VolumeConfig,
    #[serde(default)]
    pub guardrails: GuardrailConfig,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

impl ReflexConfig {
    pub fn default_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join(".config/reflex/config.toml"))
    }

    pub fn load_from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ReflexConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load() -> Self {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                if let Ok(config) = Self::load_from_path(&path) {
                    return config;
                }
            }
        }
        Self::default()
    }
}
