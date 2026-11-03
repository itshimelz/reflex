//! In-process notification router directing alerts and status updates to the Reflex HUD pill.

use crate::hud_broadcast::HudBroadcaster;
use std::sync::Arc;

pub struct NotificationDriver {
    hud: Arc<HudBroadcaster>,
}

impl NotificationDriver {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let hud = Arc::new(HudBroadcaster::new());
        Ok(Self { hud })
    }

    pub fn with_hud(hud: Arc<HudBroadcaster>) -> Self {
        Self { hud }
    }

    /// Display an in-place updating HUD pill (non-blocking, 0μs latency for caller)
    pub fn notify(&self, summary: &str, body: &str, icon: &str, timeout_ms: i32) {
        self.hud.notify(summary, body, icon, timeout_ms);
    }
}
