pub mod alarm;
pub mod battery;
pub mod call;
pub mod camera;
pub mod contacts;
pub mod location;
pub mod notification;
pub mod reminder;
pub mod sms;
pub mod timer;

use std::collections::HashMap;

/// Action registry — maps intent names to handler metadata.
/// Used for introspection and help display.
pub struct Registry {
    handlers: HashMap<String, String>,
}

impl Registry {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        handlers.insert("timer".into(), "Set/cancel timers".into());
        handlers.insert("alarm".into(), "Set alarms".into());
        handlers.insert("reminder".into(), "Set reminders".into());
        #[cfg(feature = "termux")]
        handlers.insert("sms".into(), "Send text messages".into());
        #[cfg(feature = "termux")]
        handlers.insert("call".into(), "Make phone calls".into());
        #[cfg(feature = "termux")]
        handlers.insert("camera".into(), "Take photos".into());
        #[cfg(feature = "termux")]
        handlers.insert("battery".into(), "Check battery status".into());
        #[cfg(feature = "termux")]
        handlers.insert("location".into(), "Get current location".into());
        #[cfg(feature = "termux")]
        handlers.insert("notification".into(), "Notifications and clipboard".into());
        Self { handlers }
    }

    pub fn list(&self) -> &HashMap<String, String> {
        &self.handlers
    }
}

/// Execute a shell command via termux-api and return its stdout.
/// Returns an error message string on failure instead of propagating.
pub async fn termux_command(cmd: &str, args: &[&str]) -> String {
    use tokio::process::Command;

    match Command::new(cmd).args(args).output().await {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("command failed: {stderr}")
            }
        }
        Err(e) => format!("failed to run {cmd}: {e}"),
    }
}

/// In-process fallback tool dispatcher for the agent loop. When the MCP
/// subprocess is unavailable (e.g. a static Android binary), the assistant
/// executes its own action handlers directly. Mirrors the tool names exposed
/// by `lai mcp` so the agent's tool schema stays consistent.
pub async fn run_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "set_timer" => {
            let seconds = args.get("seconds").and_then(|v| v.as_i64()).unwrap_or(0);
            let label = args
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if seconds <= 0 {
                return "timer needs a positive duration (seconds)".to_string();
            }
            let label_ref: Option<&str> = if label.is_empty() { None } else { Some(&label) };
            let _ = timer::set_timer(seconds as u64, label_ref).await;
            format!("Timer set for {seconds} seconds")
        }
        "set_reminder" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let when = args.get("when").and_then(|v| v.as_str()).unwrap_or("");
            reminder::set_reminder(text, when).await
        }
        #[cfg(feature = "termux")]
        "battery_status" => battery::status().await,
        #[cfg(feature = "termux")]
        "get_location" => location::get_location().await,
        #[cfg(feature = "termux")]
        "take_photo" => camera::take_photo().await,
        #[cfg(feature = "termux")]
        "set_clipboard" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            notification::set_clipboard(text).await
        }
        #[cfg(feature = "termux")]
        "get_clipboard" => notification::get_clipboard().await,
        #[cfg(feature = "termux")]
        "send_message" => {
            let contact = args.get("contact").and_then(|v| v.as_str()).unwrap_or("");
            let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
            sms::send_message(contact, message).await
        }
        #[cfg(feature = "termux")]
        "call_contact" => {
            let contact = args.get("contact").and_then(|v| v.as_str()).unwrap_or("");
            call::call_contact(contact).await
        }
        _ => format!("tool '{name}' is not available in-process"),
    }
}
