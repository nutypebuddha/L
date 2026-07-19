use super::termux_command;

pub async fn set_clipboard(text: &str) -> String {
    let result = termux_command("termux-clipboard-set", &[text]).await;
    if result.is_empty() || !result.starts_with("failed") {
        "Copied to clipboard".to_string()
    } else {
        format!("Failed to set clipboard: {result}")
    }
}

pub async fn get_clipboard() -> String {
    let result = termux_command("termux-clipboard-get", &[]).await;
    if result.is_empty() || result.starts_with("failed") {
        "Clipboard is empty".to_string()
    } else {
        format!("Clipboard: {result}")
    }
}

pub async fn show_notification(title: &str, content: &str) {
    termux_command(
        "termux-notification",
        &["--title", title, "--content", content],
    )
    .await;
}
