use super::termux_command;

pub async fn set_reminder(text: &str, when: &str) -> String {
    let _result = termux_command(
        "termux-notification",
        &[
            "--id",
            "lai-reminder",
            "--title",
            "Reminder",
            "--content",
            text,
            "--sound",
            "--vibrate",
            "200,400,200",
            "--on-delete",
            "termux-notification-remove lai-reminder",
        ],
    )
    .await;

    format!("Reminder set: \"{text}\" for {when}")
}
