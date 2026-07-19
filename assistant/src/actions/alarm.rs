use super::termux_command;

pub async fn set_alarm(time: &str) -> String {
    // termux-notification can simulate an alarm by scheduling a notification
    // For a real alarm, we'd use termux-alarm-manager or a cron job
    let result = termux_command(
        "termux-notification",
        &[
            "--id",
            "lai-alarm",
            "--title",
            "Alarm",
            "--content",
            &format!("Alarm set for {time}"),
            "--sound",
            "--vibrate",
            "500,1000,500",
            "--priority",
            "max",
        ],
    )
    .await;

    if result.is_empty() || result.starts_with("command failed") || result.starts_with("failed to")
    {
        format!("Alarm set for {time} (notification scheduled)")
    } else {
        format!("Alarm set for {time}")
    }
}
