//! DesktopContext: Zero-LLM Computer-Use driver for scrolling, focusing, and UI interactions.

use reflex_core::ast::GuiScrollDirection;
use reflex_core::registry::fuzzy_match_element;
use serde::Deserialize;
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct AgentEnvelope {
    #[allow(dead_code)]
    ok: bool,
    data: Option<SnapshotData>,
    #[allow(dead_code)]
    error: Option<AgentError>,
}

#[derive(Debug, Deserialize)]
struct AgentError {
    #[allow(dead_code)]
    code: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SnapshotData {
    elements: Option<Vec<UiElement>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiElement {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub role: Option<String>,
    pub description: Option<String>,
}

#[derive(Default)]
pub struct DesktopContext {
    at_spi_bus: Option<String>,
}

fn sanitize_app_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("brave") {
        "brave".to_string()
    } else if lower.contains("chrome") {
        "google-chrome".to_string()
    } else if lower.contains("firefox") {
        "firefox".to_string()
    } else if lower.contains("code") {
        "code".to_string()
    } else if let Some((prefix, _)) = raw.split_once('-') {
        prefix.to_string()
    } else {
        raw.to_string()
    }
}

impl DesktopContext {
    pub fn new() -> Self {
        // Discover AT-SPI bus if not set in environment
        let bus = std::env::var("AT_SPI_BUS_ADDRESS").ok().or_else(|| {
            // Check common Linux runtime paths under /run/user/
            for uid in &["1000", "1001", "1002"] {
                for bus_idx in &["1", "0", "2"] {
                    let path_str = format!("/run/user/{}/at-spi/bus_{}", uid, bus_idx);
                    if Path::new(&path_str).exists() {
                        return Some(format!("unix:path={}", path_str));
                    }
                }
            }
            None
        });

        Self { at_spi_bus: bus }
    }

    fn apply_bus_env(&self, cmd: &mut Command) {
        if let Some(ref bus) = self.at_spi_bus {
            cmd.env("AT_SPI_BUS_ADDRESS", bus);
        }
    }

    /// High-speed scroll: dispatches smooth directional scrolling via Wayland virtual keys
    pub async fn scroll(
        &self,
        direction: GuiScrollDirection,
        amount: u32,
        _app_class: Option<&str>,
    ) -> Result<String, String> {
        let dir_str = match direction {
            GuiScrollDirection::Up => "up",
            GuiScrollDirection::Down => "down",
            GuiScrollDirection::Left => "left",
            GuiScrollDirection::Right => "right",
        };

        // For large scrolls (>5), use Page_Up/Page_Down; for smaller steps, use directional keys
        let key = match direction {
            GuiScrollDirection::Up => {
                if amount > 5 {
                    "Page_Up"
                } else {
                    "Up"
                }
            }
            GuiScrollDirection::Down => {
                if amount > 5 {
                    "Page_Down"
                } else {
                    "Down"
                }
            }
            GuiScrollDirection::Left => "Left",
            GuiScrollDirection::Right => "Right",
        };

        let repeat_count = if amount > 5 {
            (amount / 4).max(1)
        } else {
            amount
        };

        let mut success = false;
        for _ in 0..repeat_count {
            let status = Command::new("wtype").args(["-k", key]).status().await;
            if let Ok(s) = status {
                if s.success() {
                    success = true;
                }
            }
        }

        if success {
            Ok(format!("Scrolled {} ({} lines)", dir_str, amount))
        } else {
            Err(format!("Failed to scroll {}", dir_str))
        }
    }

    /// Focus an input, field, or element in the active window
    pub async fn focus(&self, target: &str, app_class: Option<&str>) -> Result<String, String> {
        self.resolve_and_act("focus", target, app_class).await
    }

    /// Click a button, link, or element in the active window
    pub async fn click(&self, target: &str, app_class: Option<&str>) -> Result<String, String> {
        self.resolve_and_act("click", target, app_class).await
    }

    /// Press a virtual key combination in the active window
    pub async fn press_key(&self, combo: &str) -> Result<String, String> {
        let lower = combo.to_lowercase();
        let key_name = match lower.as_str() {
            "enter" | "return" => "Return",
            "escape" | "esc" => "Escape",
            "tab" => "Tab",
            "backspace" => "BackSpace",
            "space" => "space",
            "delete" => "Delete",
            "up" => "Up",
            "down" => "Down",
            "left" => "Left",
            "right" => "Right",
            "pageup" | "page_up" => "Page_Up",
            "pagedown" | "page_down" => "Page_Down",
            _ => combo,
        };

        let status = Command::new("wtype")
            .args(["-k", key_name])
            .status()
            .await
            .map_err(|e| format!("wtype execution failed: {}", e))?;

        if status.success() {
            Ok(format!("Pressed key {}", key_name))
        } else {
            Err(format!("Failed to press key {}", key_name))
        }
    }

    /// Set value or type text into active focused element
    pub async fn set_value(
        &self,
        _target: &str,
        value: &str,
        _app_class: Option<&str>,
    ) -> Result<String, String> {
        let status = Command::new("wtype")
            .args(["-s", "2", value])
            .status()
            .await
            .map_err(|e| format!("wtype typing failed: {}", e))?;

        if status.success() {
            Ok(format!("Typed {} chars", value.chars().count()))
        } else {
            Err("Failed to type value".to_string())
        }
    }

    /// One-shot search: focus search box, type query, and press Enter
    pub async fn search(&self, query: &str, app_class: Option<&str>) -> Result<String, String> {
        let is_web = app_class.is_some_and(|a| {
            let al = a.to_lowercase();
            al.contains("brave")
                || al.contains("chrome")
                || al.contains("firefox")
                || al.contains("youtube")
        });

        if is_web {
            // 1. Focus search on YouTube / Web with '/'
            let _ = Command::new("wtype").args(["-k", "slash"]).status().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
            // 2. Select all in case there is old text
            let _ = Command::new("wtype")
                .args(["-M", "ctrl", "-k", "a", "-m", "ctrl"])
                .status()
                .await;
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            // 3. Type search query
            let _ = Command::new("wtype").args(["-s", "2", query]).status().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
            // 4. Press Return to execute
            let _ = Command::new("wtype").args(["-k", "Return"]).status().await;
        } else {
            // General desktop: Ctrl+F, type, Return
            let _ = Command::new("wtype")
                .args(["-M", "ctrl", "-k", "f", "-m", "ctrl"])
                .status()
                .await;
            tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
            let _ = Command::new("wtype").args(["-s", "2", query]).status().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
            let _ = Command::new("wtype").args(["-k", "Return"]).status().await;
        }

        Ok(format!("Searched for '{}'", query))
    }

    /// Resolves an element via agent-desktop and executes an action
    async fn resolve_and_act(
        &self,
        action: &str,
        target: &str,
        app_class: Option<&str>,
    ) -> Result<String, String> {
        // 1. Try snapshot with sanitized app name first
        let clean_app = app_class.map(sanitize_app_name);
        let mut cmd = Command::new("agent-desktop");
        cmd.args(["snapshot", "-i", "--compact"]);
        if let Some(ref app) = clean_app {
            cmd.args(["--app", app]);
        }
        self.apply_bus_env(&mut cmd);

        let mut output = cmd.output().await.ok();

        // If app-specific snapshot failed, retry without --app (snapshot focused window)
        if output.as_ref().is_none_or(|o| !o.status.success()) {
            let mut fallback_cmd = Command::new("agent-desktop");
            fallback_cmd.args(["snapshot", "-i", "--compact"]);
            self.apply_bus_env(&mut fallback_cmd);
            if let Ok(fb_out) = fallback_cmd.output().await {
                if fb_out.status.success() {
                    output = Some(fb_out);
                }
            }
        }

        let envelope: Option<AgentEnvelope> = output
            .filter(|o| o.status.success())
            .and_then(|o| serde_json::from_slice(&o.stdout).ok());

        let elements = envelope.and_then(|d| d.data).and_then(|d| d.elements).unwrap_or_default();

        // 2. If interactive elements found, try fuzzy matching
        if !elements.is_empty() {
            let mut best_match: Option<(&UiElement, u32)> = None;
            for el in &elements {
                let candidate_name = el.name.as_deref().unwrap_or("");
                if let Some(score) = fuzzy_match_element(target, candidate_name) {
                    if best_match
                        .as_ref()
                        .is_none_or(|(_, best_score)| score > *best_score)
                    {
                        best_match = Some((el, score));
                    }
                } else if let Some(ref desc) = el.description {
                    if let Some(score) = fuzzy_match_element(target, desc) {
                        if best_match
                            .as_ref()
                            .is_none_or(|(_, best_score)| score > *best_score)
                        {
                            best_match = Some((el, score));
                        }
                    }
                }
            }

            if let Some((target_el, score)) = best_match {
                if score >= 60 {
                    let mut act_cmd = Command::new("agent-desktop");
                    act_cmd.args([action, &target_el.ref_id]);
                    self.apply_bus_env(&mut act_cmd);

                    let act_output = act_cmd
                        .output()
                        .await
                        .map_err(|e| format!("Failed to dispatch {}: {}", action, e))?;
                    if act_output.status.success() {
                        let matched_label = target_el.name.as_deref().unwrap_or(target);
                        return Ok(format!(
                            "{} on '{}' ({})",
                            action, matched_label, target_el.ref_id
                        ));
                    }
                }
            }
        }

        // 3. Deterministic Shortcut Fallback:
        // If accessibility snapshot returned no elements (e.g. browser without AT-SPI flag),
        // use native web/desktop shortcuts for common intentions
        let target_lower = target.to_lowercase();
        if action == "focus" && (target_lower.contains("search") || target_lower.contains("find")) {
            let is_web = app_class.is_some_and(|a| {
                let al = a.to_lowercase();
                al.contains("brave") || al.contains("chrome") || al.contains("firefox") || al.contains("youtube")
            });

            if is_web {
                // In YouTube, GitHub, Reddit, Wikipedia, Twitter: '/' is the native shortcut to focus search
                let status = Command::new("wtype").args(["-k", "slash"]).status().await;
                if let Ok(s) = status {
                    if s.success() {
                        return Ok("Focused search bar via web shortcut (/)".to_string());
                    }
                }
            } else {
                // In desktop editors/terminals: Ctrl+F focuses search
                let status = Command::new("wtype").args(["-M", "ctrl", "-k", "f", "-m", "ctrl"]).status().await;
                if let Ok(s) = status {
                    if s.success() {
                        return Ok("Focused search via shortcut (Ctrl+F)".to_string());
                    }
                }
            }
        }

        Err(format!(
            "No element matching '{}' met confidence threshold",
            target
        ))
    }
}
