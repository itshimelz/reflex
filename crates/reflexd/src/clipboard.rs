//! Guardrailed Wayland clipboard utilities and virtual keystroke injection.

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub const MAX_CLIPBOARD_TYPE_LEN: usize = 500;

#[derive(Default)]
pub struct ClipboardDriver;

impl ClipboardDriver {
    pub fn new() -> Self {
        Self
    }

    /// Read raw text from Wayland clipboard via wl-paste
    pub async fn read_text(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("wl-paste")
            .arg("--no-newline")
            .output()
            .await?;

        if !output.status.success() {
            // Nothing copied or clipboard is empty
            return Ok(String::new());
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(text)
    }

    /// Type current clipboard text using wtype with length and trailing-newline guardrails
    pub async fn type_clipboard(&self) -> Result<usize, String> {
        let raw = self
            .read_text()
            .await
            .map_err(|e| format!("Clipboard read failed: {}", e))?;

        if raw.is_empty() {
            return Err("Clipboard is empty".to_string());
        }

        let len = raw.chars().count();
        if len > MAX_CLIPBOARD_TYPE_LEN {
            return Err(format!(
                "Exceeds safety limit: {} characters (max {})",
                len, MAX_CLIPBOARD_TYPE_LEN
            ));
        }

        // Sanitize trailing newlines to prevent accidental shell command execution
        let sanitized = raw.trim_end_matches(['\r', '\n']);
        if sanitized.is_empty() {
            return Err("Clipboard content contains only newlines".to_string());
        }

        let mut child = Command::new("wtype")
            .arg("-d")
            .arg("2") // 2ms inter-key delay for terminal stability
            .arg("--")
            .arg(sanitized)
            .spawn()
            .map_err(|e| format!("Failed to spawn wtype: {}", e))?;

        let status = child
            .wait()
            .await
            .map_err(|e| format!("wtype failed: {}", e))?;

        if !status.success() {
            return Err("wtype failed during keystroke injection".to_string());
        }

        Ok(sanitized.chars().count())
    }

    /// Clear Wayland clipboard
    pub async fn clear(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = Command::new("wl-copy").arg("--clear").status().await?;
        if !status.success() {
            return Err("wl-copy --clear failed".into());
        }
        Ok(())
    }

    /// Strip non-plain MIME types and re-copy as plain text
    pub async fn strip_formatting(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let text = self.read_text().await?;
        if text.is_empty() {
            return Ok(0);
        }
        let len = text.chars().count();
        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg("text/plain;charset=utf-8")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
            stdin.flush().await?;
        }

        child.wait().await?;
        Ok(len)
    }
}
