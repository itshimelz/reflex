//! Guardrails and safety policy state machine.

use crate::registry::HyprlandClient;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardrailDecision {
    Allowed,
    VelocityClamped {
        requested: u8,
        clamped: u8,
    },
    CeilingClamped {
        requested: u8,
        clamped: u8,
    },
    RequiresConfirmation {
        target_address: String,
        window_class: String,
    },
}

#[derive(Debug, Clone)]
pub struct PendingConfirmation {
    pub target_address: String,
    pub window_class: String,
    pub expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct GuardrailEngine {
    pub volume_ceiling: u8,
    pub max_step_velocity: u8,
    pub velocity_window: Duration,
    pub unkillable_classes: HashSet<String>,
    pub confirmation_timeout: Duration,

    pub current_volume: u8,
    pub last_volume_time: Instant,
    pub pending_confirmation: Option<PendingConfirmation>,
}

impl Default for GuardrailEngine {
    fn default() -> Self {
        let mut unkillable = HashSet::new();
        unkillable.insert("kitty".to_string());
        unkillable.insert("neovim".to_string());
        unkillable.insert("nvim".to_string());
        unkillable.insert("brave".to_string());
        unkillable.insert("brave-browser".to_string());
        unkillable.insert("code".to_string());
        unkillable.insert("vscode".to_string());
        unkillable.insert("antigravity".to_string());

        Self {
            volume_ceiling: 80,
            max_step_velocity: 15,
            velocity_window: Duration::from_millis(500),
            unkillable_classes: unkillable,
            confirmation_timeout: Duration::from_secs(4),

            current_volume: 50,
            last_volume_time: Instant::now() - Duration::from_secs(10),
            pending_confirmation: None,
        }
    }
}

impl GuardrailEngine {
    /// Evaluate an absolute volume set target (0..=100)
    pub fn evaluate_volume_set(&mut self, requested: u8) -> (u8, GuardrailDecision) {
        let now = Instant::now();
        let mut target = requested;
        let mut decision = GuardrailDecision::Allowed;

        // 1. Check positive velocity limit
        if target > self.current_volume {
            let max_allowed = self.current_volume.saturating_add(self.max_step_velocity);
            if target > max_allowed {
                target = max_allowed;
                decision = GuardrailDecision::VelocityClamped {
                    requested,
                    clamped: target,
                };
            }
        }

        // 2. Check hard ceiling
        if target > self.volume_ceiling {
            target = self.volume_ceiling;
            decision = GuardrailDecision::CeilingClamped {
                requested,
                clamped: target,
            };
        }

        self.current_volume = target;
        self.last_volume_time = now;
        (target, decision)
    }

    /// Evaluate a relative volume step (-100..=100)
    pub fn evaluate_volume_step(&mut self, step: i8) -> (i8, GuardrailDecision) {
        let current = self.current_volume as i16;
        let requested_target = (current + step as i16).clamp(0, 100) as u8;
        let (actual_target, decision) = self.evaluate_volume_set(requested_target);
        let actual_step = actual_target as i16 - current;
        (actual_step as i8, decision)
    }

    /// Evaluate closing a window
    pub fn evaluate_window_close(&mut self, client: &HyprlandClient) -> GuardrailDecision {
        let class_lower = client.class.to_lowercase();
        let is_unkillable = self
            .unkillable_classes
            .iter()
            .any(|u| class_lower.contains(&u.to_lowercase()));

        if is_unkillable {
            self.pending_confirmation = Some(PendingConfirmation {
                target_address: client.address.clone(),
                window_class: client.class.clone(),
                expires_at: Instant::now() + self.confirmation_timeout,
            });

            GuardrailDecision::RequiresConfirmation {
                target_address: client.address.clone(),
                window_class: client.class.clone(),
            }
        } else {
            GuardrailDecision::Allowed
        }
    }

    /// Confirm pending protected action
    pub fn confirm(&mut self) -> Option<String> {
        let pending = self.pending_confirmation.take()?;
        if Instant::now() <= pending.expires_at {
            Some(pending.target_address)
        } else {
            None
        }
    }
}
