/// Proactive suggestion engine — monitors context and suggests actions
/// without being asked.
///
/// Monitors:
/// - Time-based: morning greetings, evening wind-down, meeting prep
/// - Location-based: arriving at work/home, nearby places
/// - Usage patterns: frequent contacts, common queries
/// - Device state: low battery warnings, charging state
use chrono::{Local, Timelike};

pub struct ProactiveEngine {
    last_suggestion: Option<chrono::DateTime<Local>>,
    cooldown_minutes: i64,
}

impl ProactiveEngine {
    pub fn new() -> Self {
        Self {
            last_suggestion: None,
            cooldown_minutes: 30,
        }
    }

    /// Check if there's a proactive suggestion to offer right now.
    /// Returns None if nothing relevant, or Some(suggestion text).
    pub fn check(&mut self) -> Option<String> {
        let now = Local::now();

        // Cooldown check
        if let Some(last) = self.last_suggestion {
            let diff = now.signed_duration_since(last);
            if diff.num_minutes() < self.cooldown_minutes {
                return None;
            }
        }

        let hour = now.hour();

        let suggestion = match hour {
            // Morning: 6-9am
            6..=9 => Some(
                "Good morning! Would you like me to check your schedule or the weather?"
                    .to_string(),
            ),
            // Lunch: 11:30am-1pm
            11..=13 => Some("It's lunchtime. Want me to set a reminder for anything?".to_string()),
            // Evening: 5-7pm
            17..=19 => {
                Some("Heading home? Want me to set a reminder for anything tonight?".to_string())
            }
            // Late night: 10pm-12am
            22..=23 => Some("It's getting late. Want me to set an alarm for tomorrow?".to_string()),
            _ => None,
        };

        if suggestion.is_some() {
            self.last_suggestion = Some(now);
        }

        suggestion
    }
}
