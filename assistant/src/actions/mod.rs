pub mod alarm;
pub mod battery;
pub mod call;
pub mod camera;
pub mod contacts;
pub mod location;
pub mod memory;
pub mod notification;
pub mod reminder;
pub mod schedule;
pub mod sms;
pub mod timer;
#[cfg(feature = "web")]
pub mod web;

use std::collections::HashMap;

/// Action registry — maps intent names to handler metadata.
/// Used for introspection and help display.
pub struct Registry {
    handlers: HashMap<String, String>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        handlers.insert("timer".into(), "Set/cancel timers".into());
        handlers.insert("alarm".into(), "Set alarms".into());
        handlers.insert("reminder".into(), "Set reminders".into());
        handlers.insert("memory".into(), "Remember/recall/forget user facts".into());
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
        #[cfg(feature = "web")]
        handlers.insert("web".into(), "Web search, fetch, and HTTP requests".into());
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
        "remember" => {
            let (key, value) = remember_args(args);
            memory::remember(&key, &value).await
        }
        "recall" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            memory::recall(key).await
        }
        "forget" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            memory::forget(key).await
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
        #[cfg(feature = "web")]
        "web_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            web::search(query).await
        }
        #[cfg(feature = "web")]
        "web_fetch" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            web::fetch(url).await
        }
        #[cfg(feature = "web")]
        "http_request" => {
            let method = args.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let body = args
                .get("body")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            web::http_request(method, url, body).await
        }
        _ => format!("tool '{name}' is not available in-process"),
    }
}

/// Extract `(key, value)` for the `remember` tool, tolerating the shapes small
/// models commonly emit instead of the declared `{key, value}` object:
///
///   * `{"key":"x","value":"y"}` — the schema shape
///   * `{"x":"y"}` — a single arbitrary pair (qwen2.5:0.5b collapses key/value
///     into one field)
///   * `{"value":"y"}` / `{"key":"x"}` — partial; the lone field is kept and the
///     missing half stays empty so the handler still fails loud
///
/// This never fabricates data: an empty or non-object argument yields empty
/// strings, which `memory::remember` rejects with a clear message.
fn remember_args(args: &serde_json::Value) -> (String, String) {
    let get = |k: &str| {
        args.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let key = get("key");
    let value = get("value");
    if !key.is_empty() && !value.is_empty() {
        return (key, value);
    }
    // Recover the single-arbitrary-pair shape: pick the first string-valued
    // field that is not the reserved `key`/`value` names.
    if key.is_empty() && value.is_empty() {
        if let Some(map) = args.as_object() {
            // Determinism: iterate keys in sorted order so recovery is stable.
            let mut pairs: Vec<(&String, &serde_json::Value)> = map.iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(b.0));
            for (k, v) in pairs {
                if k == "key" || k == "value" {
                    continue;
                }
                if let Some(s) = v.as_str() {
                    return (k.clone(), s.to_string());
                }
            }
        }
    }
    (key, value)
}

#[cfg(test)]
mod tests {
    use super::remember_args;
    use serde_json::json;

    #[test]
    fn remember_args_schema_shape() {
        let (k, v) = remember_args(&json!({"key": "city", "value": "Berlin"}));
        assert_eq!((k.as_str(), v.as_str()), ("city", "Berlin"));
    }

    #[test]
    fn remember_args_single_pair_recovered() {
        let (k, v) = remember_args(&json!({"favorite_language": "Rust"}));
        assert_eq!((k.as_str(), v.as_str()), ("favorite_language", "Rust"));
    }

    #[test]
    fn remember_args_single_pair_is_deterministic() {
        let (k, _) = remember_args(&json!({"zebra": "1", "apple": "2"}));
        assert_eq!(k, "apple");
    }

    #[test]
    fn remember_args_empty_stays_empty() {
        let (k, v) = remember_args(&json!({}));
        assert!(k.is_empty() && v.is_empty());
    }

    #[test]
    fn remember_args_partial_key_only() {
        let (k, v) = remember_args(&json!({"key": "city"}));
        assert_eq!(k, "city");
        assert!(v.is_empty());
    }
}
