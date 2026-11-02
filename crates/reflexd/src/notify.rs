//! D-Bus notification client for QuickShell / Freedesktop OSD toasts.

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::Duration;
use zbus::zvariant::Value;
use zbus::Connection;

pub struct NotificationDriver {
    dbus: Connection,
    last_notification_id: Arc<AtomicU32>,
}

impl NotificationDriver {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dbus = Connection::session().await?;
        Ok(Self {
            dbus,
            last_notification_id: Arc::new(AtomicU32::new(0)),
        })
    }

    /// Display an in-place updating OSD toast (non-blocking, 0μs latency for caller)
    pub fn notify(&self, summary: &str, body: &str, icon: &str, timeout_ms: i32) {
        let dbus = self.dbus.clone();
        let last_id = Arc::clone(&self.last_notification_id);
        let summary = summary.to_string();
        let body = body.to_string();
        let icon = icon.to_string();

        tokio::spawn(async move {
            let app_name = "reflex";
            let replaces_id = last_id.load(std::sync::atomic::Ordering::Relaxed);
            let actions: Vec<&str> = Vec::new();
            let mut hints: HashMap<&str, Value> = HashMap::new();
            hints.insert("transient", Value::Bool(true));
            hints.insert("urgency", Value::U8(1));

            let res = tokio::time::timeout(
                Duration::from_millis(150),
                dbus.call_method(
                    Some("org.freedesktop.Notifications"),
                    "/org/freedesktop/Notifications",
                    Some("org.freedesktop.Notifications"),
                    "Notify",
                    &(
                        app_name,
                        replaces_id,
                        icon.as_str(),
                        summary.as_str(),
                        body.as_str(),
                        actions,
                        hints,
                        timeout_ms,
                    ),
                ),
            )
            .await;

            if let Ok(Ok(msg)) = res {
                if let Ok(new_id) = msg.body().deserialize::<u32>() {
                    last_id.store(new_id, std::sync::atomic::Ordering::Relaxed);
                }
            }
        });
    }
}
