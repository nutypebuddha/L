use super::termux_command;

pub async fn status() -> String {
    let output = termux_command("termux-battery-status", &[]).await;

    if output.is_empty() || output.starts_with("failed") {
        return "Failed to get battery status.".to_string();
    }

    // Parse JSON: {"percentage": 85, "status": "CHARGING", "health": "GOOD", ...}
    match serde_json::from_str::<serde_json::Value>(&output) {
        Ok(json) => {
            let pct = json["percentage"].as_i64().unwrap_or(0);
            let status = json["status"].as_str().unwrap_or("unknown");
            let health = json["health"].as_str().unwrap_or("unknown");

            format!("Battery: {pct}% ({status}), Health: {health}")
        }
        Err(_) => format!("Battery info: {output}"),
    }
}
