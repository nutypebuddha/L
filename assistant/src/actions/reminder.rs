use super::schedule::{self, Kind};
use super::termux_command;
use crate::memory::data_dir;

pub async fn set_reminder(text: &str, when: &str) -> String {
    // Persist the reminder so it survives a daemon restart. If the "when"
    // string parses to a relative delay, schedule a durable fire; otherwise
    // fall back to the immediate notification only.
    if let Some(secs) = parse_relative_secs(when) {
        let dir = data_dir();
        // Stable id: the reminder text, so re-issuing the same reminder
        // replaces rather than duplicates.
        if let Err(e) = schedule::add(&dir, Kind::Reminder, text, secs, text).await {
            return format!("could not persist reminder: {e}");
        }
        let dir_moved = dir.clone();
        let text_owned = text.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
            fire_notification(&text_owned).await;
            let _ = schedule::remove(&dir_moved, &Kind::Reminder, &text_owned).await;
        });
        return format!("Reminder set: \"{text}\" for {when}");
    }

    // Unparseable time — fire the notification now as a best-effort cue.
    fire_notification(text).await;
    format!("Reminder set: \"{text}\" for {when}")
}

async fn fire_notification(text: &str) {
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
}

/// Parse a relative time phrase into seconds from now. Handles the common
/// forms the assistant sees: "in 5 minutes", "30 seconds", "2 hours",
/// "in 1 hour", "90s", "10m", "1h". Returns `None` for absolute times or
/// anything it can't confidently interpret — the caller then degrades to an
/// immediate notification rather than guessing a fire time.
pub fn parse_relative_secs(when: &str) -> Option<u64> {
    let w = when.trim().to_lowercase();
    let w = w.strip_prefix("in ").unwrap_or(&w).trim();

    // Compact forms: "90s", "10m", "1h".
    if let Some(num) = w
        .strip_suffix('s')
        .and_then(|n| n.trim().parse::<u64>().ok())
    {
        return Some(num);
    }
    if let Some(num) = w
        .strip_suffix('m')
        .and_then(|n| n.trim().parse::<u64>().ok())
    {
        return Some(num * 60);
    }
    if let Some(num) = w
        .strip_suffix('h')
        .and_then(|n| n.trim().parse::<u64>().ok())
    {
        return Some(num * 3600);
    }

    // Worded forms: "<n> <unit>".
    let mut parts = w.split_whitespace();
    let n: u64 = parts.next()?.parse().ok()?;
    let unit = parts.next()?;
    let mult = match unit.trim_end_matches('s') {
        "second" | "sec" => 1,
        "minute" | "min" => 60,
        "hour" | "hr" => 3600,
        "day" => 86_400,
        _ => return None,
    };
    Some(n * mult)
}

#[cfg(test)]
mod tests {
    use super::parse_relative_secs;

    #[test]
    fn worded_forms() {
        assert_eq!(parse_relative_secs("in 5 minutes"), Some(300));
        assert_eq!(parse_relative_secs("30 seconds"), Some(30));
        assert_eq!(parse_relative_secs("2 hours"), Some(7200));
        assert_eq!(parse_relative_secs("in 1 hour"), Some(3600));
        assert_eq!(parse_relative_secs("1 day"), Some(86_400));
    }

    #[test]
    fn compact_forms() {
        assert_eq!(parse_relative_secs("90s"), Some(90));
        assert_eq!(parse_relative_secs("10m"), Some(600));
        assert_eq!(parse_relative_secs("1h"), Some(3600));
    }

    #[test]
    fn unparseable_returns_none() {
        assert_eq!(parse_relative_secs("tomorrow at noon"), None);
        assert_eq!(parse_relative_secs("next week"), None);
        assert_eq!(parse_relative_secs(""), None);
    }
}
