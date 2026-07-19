/// Multimodal input — camera photos and screen context.
///
/// Uses termux-api for camera access and screenshot capture.
use crate::actions::termux_command;

pub async fn take_photo_for_analysis() -> anyhow::Result<String> {
    let path = "/tmp/lai-analysis.jpg";
    let result = termux_command("termux-camera-photo", &[path]).await;

    if result.starts_with("failed") {
        return Err(anyhow::anyhow!("Failed to take photo: {result}"));
    }

    Ok(path.to_string())
}

pub async fn take_screenshot() -> anyhow::Result<String> {
    // termux doesn't have a native screenshot command,
    // but we can use termux-toast to show a message
    // and read screen content via accessibility services
    // For now, return a placeholder
    Err(anyhow::anyhow!(
        "Screen capture not yet implemented on this device"
    ))
}
