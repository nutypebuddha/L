use super::termux_command;

pub async fn get_location() -> String {
    let output = termux_command("termux-location", &["-p", "network", "-r", "once"]).await;

    if output.is_empty() || output.starts_with("failed") {
        return "Failed to get location. Make sure location permission is granted.".to_string();
    }

    match serde_json::from_str::<serde_json::Value>(&output) {
        Ok(json) => {
            let lat = json["latitude"].as_f64().unwrap_or(0.0);
            let lon = json["longitude"].as_f64().unwrap_or(0.0);
            let acc = json["accuracy"].as_f64().unwrap_or(0.0);

            format!("Location: {lat:.4}, {lon:.4} (accuracy: {acc:.0}m)")
        }
        Err(_) => format!("Location: {output}"),
    }
}
