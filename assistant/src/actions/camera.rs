use super::termux_command;

pub async fn take_photo() -> String {
    let result = termux_command("termux-camera-photo", &["/sdcard/Download/lai-photo.jpg"]).await;

    if result.is_empty() || result.starts_with("failed") {
        // Try alternative path
        let result2 = termux_command("termux-camera-photo", &["/tmp/lai-photo.jpg"]).await;

        if result2.is_empty() || result2.starts_with("failed") {
            "Failed to take photo. Make sure camera permission is granted.".to_string()
        } else {
            "Photo saved to /tmp/lai-photo.jpg".to_string()
        }
    } else {
        "Photo saved to /sdcard/Download/lai-photo.jpg".to_string()
    }
}
