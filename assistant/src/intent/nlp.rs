use super::schema::Intent;

/// Lightweight keyword-based intent classifier.
///
/// This is a deterministic, offline-first classifier that uses keyword
/// matching and simple pattern rules. No ML model required.
///
/// Priority order: goodbye > help > system actions > ecosystem intents >
///   timer > alarm > reminder > sms > call > camera > battery > location >
///   clipboard > query > conversational
pub fn classify(text: &str) -> Intent {
    let lower = text.to_lowercase();
    let lower = lower.trim();

    // ── Goodbye ──────────────────────────────────────────────
    if matches!(
        lower,
        "goodbye" | "bye" | "see you" | "see ya" | "talk later" | "exit" | "quit"
    ) {
        return Intent::Goodbye;
    }

    // ── Help ─────────────────────────────────────────────────
    if lower == "help" || lower == "what can you do" || lower == "what can you help with" {
        return Intent::Help;
    }

    // ── Ecosystem intents (checked before system actions) ────
    if let Some(intent) = parse_convert(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_eval(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_validate(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_score(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_fix(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_solve(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_formula(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_search_formulas(lower) {
        return intent;
    }
    if let Some(intent) = parse_chain_formulas(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_traverse(lower) {
        return intent;
    }
    if let Some(intent) = parse_classify(lower) {
        return intent;
    }
    if let Some(intent) = parse_wheel(lower) {
        return intent;
    }
    if let Some(intent) = parse_reason(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_shikai(lower, text) {
        return intent;
    }
    if let Some(intent) = parse_bankai(lower, text) {
        return intent;
    }

    // ── System actions ───────────────────────────────────────

    // Timer
    if let Some(intent) = parse_timer(lower) {
        return intent;
    }

    // Alarm
    if let Some(intent) = parse_alarm(lower) {
        return intent;
    }

    // Reminder
    if let Some(intent) = parse_reminder(lower) {
        return intent;
    }

    // Send message / SMS — Termux-only action; not advertised without the
    // `termux` feature so the capability list matches what the device can do.
    #[cfg(feature = "termux")]
    if let Some(intent) = parse_sms(lower, text) {
        return intent;
    }

    // Call — Termux-only
    #[cfg(feature = "termux")]
    if let Some(intent) = parse_call(lower, text) {
        return intent;
    }

    // Take photo — Termux-only
    #[cfg(feature = "termux")]
    if lower.contains("take a photo")
        || lower.contains("take a picture")
        || lower.contains("snap a photo")
        || lower.contains("snap a picture")
        || lower.contains("take photo")
    {
        return Intent::TakePhoto;
    }

    // Battery — Termux-only
    #[cfg(feature = "termux")]
    if lower.contains("battery") || lower.contains("charge") || lower.contains("power level") {
        return Intent::BatteryStatus;
    }

    // Location — Termux-only
    #[cfg(feature = "termux")]
    if lower.contains("where am i")
        || lower.contains("my location")
        || lower.contains("what's my location")
        || lower.contains("get location")
    {
        return Intent::GetLocation;
    }

    // Clipboard — Termux-only
    #[cfg(feature = "termux")]
    if lower.starts_with("copy") && lower.contains("clipboard") {
        let text = text
            .trim_start_matches(|c: char| c.is_alphabetic() || c == ' ')
            .trim_start_matches("copy")
            .trim_start_matches("this to clipboard")
            .trim_start_matches("to clipboard")
            .trim()
            .to_string();
        return Intent::SetClipboard { text };
    }
    #[cfg(feature = "termux")]
    if lower == "read clipboard" || lower == "what's on my clipboard" || lower == "paste" {
        return Intent::GetClipboard;
    }

    // ── Query (question words) ───────────────────────────────
    if lower.starts_with("what ")
        || lower.starts_with("who ")
        || lower.starts_with("when ")
        || lower.starts_with("where ")
        || lower.starts_with("why ")
        || lower.starts_with("how ")
        || lower.starts_with("is ")
        || lower.starts_with("are ")
        || lower.starts_with("can ")
        || lower.starts_with("do ")
        || lower.starts_with("does ")
    {
        return Intent::Query {
            text: text.to_string(),
        };
    }

    // ── Conversational fallback ──────────────────────────────
    if !lower.is_empty() {
        return Intent::Conversational {
            text: text.to_string(),
        };
    }

    Intent::Unknown
}

// ── Ecosystem parsers ──────────────────────────────────────────

/// "convert 100 miles to kilometers" / "5 kg in pounds"
fn parse_convert(lower: &str, original: &str) -> Option<Intent> {
    if !lower.contains("convert") && !lower.contains(" in ") && !lower.contains(" to ") {
        return None;
    }

    // "convert 100 miles to kilometers"
    if let Some(rest) = lower.strip_prefix("convert ") {
        let parts: Vec<&str> = rest
            .splitn(3, |c: char| c.is_whitespace() || c == ',')
            .collect();
        if parts.len() >= 3 {
            if let Ok(value) = parts[0].parse::<f64>() {
                let from = parts[1].trim();
                let to = parts[2]
                    .trim()
                    .trim_start_matches("to ")
                    .trim_start_matches("in ")
                    .trim();
                return Some(Intent::Convert {
                    value,
                    from: from.to_string(),
                    to: to.to_string(),
                });
            }
        }
    }

    // "100 miles to kilometers" / "5 kg in pounds"
    let words: Vec<&str> = lower.split_whitespace().collect();
    for i in 0..words.len() {
        if let Ok(value) = words[i].parse::<f64>() {
            if i + 2 < words.len() {
                let after = words[i + 1];
                let connector = words[i + 2];
                if connector == "to" || connector == "in" {
                    let from = after.to_string();
                    let to: String = words[i + 3..].join(" ");
                    if !to.is_empty() {
                        return Some(Intent::Convert { value, from, to });
                    }
                }
            }
        }
    }

    // "convert miles to kilometers" (no number — use 1.0)
    if lower.starts_with("convert ") {
        let rest = original.trim_start_matches("convert ").trim();
        // Split on " to " or " in "
        if let Some(pos) = rest.to_lowercase().find(" to ") {
            let from = rest[..pos].trim().to_string();
            let to = rest[pos + 4..].trim().to_string();
            if !from.is_empty() && !to.is_empty() {
                return Some(Intent::Convert {
                    value: 1.0,
                    from,
                    to,
                });
            }
        }
        if let Some(pos) = rest.to_lowercase().find(" in ") {
            let from = rest[..pos].trim().to_string();
            let to = rest[pos + 4..].trim().to_string();
            if !from.is_empty() && !to.is_empty() {
                return Some(Intent::Convert {
                    value: 1.0,
                    from,
                    to,
                });
            }
        }
    }

    None
}

/// "calculate 2+2" / "what's sqrt(144)" / "compute 15*3"
fn parse_eval(lower: &str, original: &str) -> Option<Intent> {
    let triggers = [
        "calculate",
        "compute",
        "evaluate",
        "eval",
        "solve for",
        "what is",
        "what's",
    ];

    for trigger in &triggers {
        if let Some(rest) = lower.strip_prefix(trigger) {
            let expr = rest
                .trim()
                .trim_start_matches(|c: char| c == ':' || c == ' ');
            if !expr.is_empty() && looks_like_math(expr) {
                return Some(Intent::Eval {
                    expression: expr.to_string(),
                });
            }
        }
    }

    // "sqrt(144)" / "2 + 3 * 4" — bare math expression
    if looks_like_math(lower)
        && !lower.contains(' ')
        && lower.chars().any(|c| "+-*/^%()".contains(c))
    {
        return Some(Intent::Eval {
            expression: original.to_string(),
        });
    }

    None
}

/// "validate this claim" / "check if water is H2O" / "verify X against Y"
fn parse_validate(lower: &str, original: &str) -> Option<Intent> {
    // "validate X against Y"
    if lower.starts_with("validate ") {
        if let Some(rest) = lower.strip_prefix("validate ") {
            if let Some((claim, context)) = parse_against(rest) {
                return Some(Intent::Validate {
                    text: claim,
                    context,
                });
            }
            // No "against" — use claim as text, empty context
            return Some(Intent::Validate {
                text: rest.to_string(),
                context: String::new(),
            });
        }
    }

    // "check if X is Y" / "verify X"
    if lower.starts_with("check if ") || lower.starts_with("verify ") {
        let prefix_len = if lower.starts_with("check if ") {
            "check if ".len()
        } else {
            "verify ".len()
        };
        let claim = original.trim().get(prefix_len..).unwrap_or("").to_string();
        return Some(Intent::Validate {
            text: claim,
            context: String::new(),
        });
    }

    None
}

/// "score this" / "how confident" / "confidence level"
fn parse_score(lower: &str, _original: &str) -> Option<Intent> {
    if lower.starts_with("score ")
        || lower.starts_with("score this")
        || lower.contains("how confident")
        || lower.contains("confidence level")
        || lower.contains("rate this")
    {
        Some(Intent::Score {
            text: _original.to_string(),
        })
    } else {
        None
    }
}

/// "fix this response" / "auto-fix X against Y"
fn parse_fix(lower: &str, original: &str) -> Option<Intent> {
    if lower.starts_with("fix ") || lower.starts_with("auto-fix ") || lower.starts_with("autofix ")
    {
        let rest = original
            .trim()
            .trim_start_matches("fix ")
            .trim_start_matches("auto-fix ")
            .trim_start_matches("autofix ")
            .trim();
        if let Some((text, context)) = parse_against(rest) {
            return Some(Intent::Fix { text, context });
        }
        return Some(Intent::Fix {
            text: rest.to_string(),
            context: String::new(),
        });
    }
    None
}

/// "prove that 2+2=4" / "reason about X" / "solve X"
fn parse_solve(lower: &str, original: &str) -> Option<Intent> {
    if lower.starts_with("prove ")
        || lower.starts_with("reason about ")
        || lower.starts_with("reason on ")
        || lower.starts_with("reasoning about ")
    {
        let prefix_len = if lower.starts_with("reasoning ") {
            "reasoning about ".len()
        } else if lower.starts_with("reason on ") {
            "reason on ".len()
        } else {
            let first_word = lower.split_whitespace().next().unwrap_or("");
            first_word.len() + 1
        };
        let query = original.trim().get(prefix_len..).unwrap_or("").to_string();
        if !query.is_empty() {
            return Some(Intent::Solve { query });
        }
    }
    None
}

/// "evaluate the circle_area formula with r=5" / "run formula X"
fn parse_formula(lower: &str, original: &str) -> Option<Intent> {
    // Match various trigger patterns
    let triggers = [
        "evaluate formula ",
        "run formula ",
        "use formula ",
        "evaluate the formula ",
        "use the formula ",
    ];

    // First try exact prefix matches
    for trigger in &triggers {
        if lower.starts_with(trigger) {
            let rest = original.trim().trim_start_matches(trigger).trim();
            return extract_formula_with_args(rest);
        }
    }

    // Then try "evaluate the X formula" / "run the X formula" pattern
    if lower.starts_with("evaluate the ") || lower.starts_with("run the ") {
        let prefix_len = if lower.starts_with("evaluate the ") {
            "evaluate the ".len()
        } else {
            "run the ".len()
        };
        let rest = original.trim().get(prefix_len..).unwrap_or("");
        if let Some(name_end) = rest.to_lowercase().find(" formula") {
            let name = rest[..name_end].trim().to_string();
            let after_formula = rest[name_end + " formula".len()..].trim().to_string();
            if !name.is_empty() {
                if let Some((args, _rest_args)) = parse_with_args(&after_formula) {
                    return Some(Intent::Formula {
                        name,
                        args: vec![args],
                    });
                }
                return Some(Intent::Formula { name, args: vec![] });
            }
        }
    }

    None
}

fn extract_formula_with_args(rest: &str) -> Option<Intent> {
    if let Some((_args, _rest_args)) = parse_with_args(rest) {
        let name = rest.split_whitespace().next().unwrap_or("").to_string();
        // If there's "with", the name is everything before "with"
        if let Some(pos) = rest.to_lowercase().find(" with ") {
            let name = rest[..pos].trim().to_string();
            return Some(Intent::Formula {
                name,
                args: vec![rest[pos + 6..].trim().to_string()],
            });
        }
        if let Some(pos) = rest.to_lowercase().find(" using ") {
            let name = rest[..pos].trim().to_string();
            return Some(Intent::Formula {
                name,
                args: vec![rest[pos + 7..].trim().to_string()],
            });
        }
        if !name.is_empty() {
            return Some(Intent::Formula { name, args: vec![] });
        }
    }
    None
}

fn parse_with_args(text: &str) -> Option<(String, &str)> {
    if let Some(pos) = text.to_lowercase().find(" with ") {
        let args = text[pos + 6..].trim();
        return Some((text[..pos].trim().to_string(), args));
    }
    if let Some(pos) = text.to_lowercase().find(" using ") {
        let args = text[pos + 7..].trim();
        return Some((text[..pos].trim().to_string(), args));
    }
    None
}

/// "search formulas for gravity" / "find formulas about energy"
fn parse_search_formulas(lower: &str) -> Option<Intent> {
    if lower.starts_with("search formulas for ")
        || lower.starts_with("find formulas for ")
        || lower.starts_with("search formulas about ")
        || lower.starts_with("find formulas about ")
        || lower.starts_with("look up formula ")
        || lower.starts_with("look up formulas ")
    {
        let keyword = lower
            .trim_start_matches("search formulas for ")
            .trim_start_matches("find formulas for ")
            .trim_start_matches("search formulas about ")
            .trim_start_matches("find formulas about ")
            .trim_start_matches("look up formula ")
            .trim_start_matches("look up formulas ")
            .trim()
            .to_string();
        if !keyword.is_empty() {
            return Some(Intent::SearchFormulas { keyword });
        }
    }
    None
}

/// "chain formulas circle_area, sphere_volume" / "chain X and Y"
fn parse_chain_formulas(lower: &str, original: &str) -> Option<Intent> {
    if lower.starts_with("chain formulas ") || lower.starts_with("chain formula ") {
        let rest = original
            .trim()
            .trim_start_matches("chain formulas ")
            .trim_start_matches("chain formula ")
            .trim();
        // Split on " with " or " using "
        if let Some(pos) = rest.to_lowercase().find(" with ") {
            let formulas = rest[..pos].trim().to_string();
            let args = rest[pos + 6..].trim().to_string();
            return Some(Intent::ChainFormulas { formulas, args });
        }
        if let Some(pos) = rest.to_lowercase().find(" using ") {
            let formulas = rest[..pos].trim().to_string();
            let args = rest[pos + 7..].trim().to_string();
            return Some(Intent::ChainFormulas { formulas, args });
        }
        return Some(Intent::ChainFormulas {
            formulas: rest.to_string(),
            args: String::new(),
        });
    }
    None
}

/// "traverse the mangala domain" / "explore aries" / "wheel aries"
fn parse_traverse(lower: &str) -> Option<Intent> {
    let triggers = ["traverse ", "explore ", "walk ", "go through "];
    for trigger in &triggers {
        if let Some(rest) = lower.strip_prefix(trigger) {
            let domain = rest
                .trim_start_matches("the ")
                .trim_start_matches("domain ")
                .trim()
                .trim_end_matches(" domain")
                .to_string();
            if !domain.is_empty() {
                return Some(Intent::Traverse { domain, depth: 5 });
            }
        }
    }
    None
}

/// "classify mercury" / "what axis is jupiter"
fn parse_classify(lower: &str) -> Option<Intent> {
    if lower.starts_with("classify ") {
        let token = lower.strip_prefix("classify ").unwrap().trim().to_string();
        if !token.is_empty() {
            return Some(Intent::Classify { token });
        }
    }
    if lower.starts_with("what axis is ") {
        let token = lower
            .strip_prefix("what axis is ")
            .unwrap()
            .trim()
            .to_string();
        if !token.is_empty() {
            return Some(Intent::Classify { token });
        }
    }
    None
}

/// "show the wheel" / "what's on the wheel" / "wheel aries"
fn parse_wheel(lower: &str) -> Option<Intent> {
    if lower.contains("show the wheel")
        || lower.contains("show wheel")
        || lower.contains("zodiac wheel")
        || lower.contains("what's on the wheel")
        || lower.contains("wheel")
    {
        // Check if a domain is specified
        let domains = [
            "aries",
            "taurus",
            "gemini",
            "cancer",
            "leo",
            "virgo",
            "libra",
            "scorpio",
            "sagittarius",
            "capricorn",
            "aquarius",
            "pisces",
            "mangala",
            "vrishabha",
            "mithuna",
            "karka",
            "simha",
            "kanya",
            "tula",
            "vrishchika",
            "dhanu",
            "makara",
            "kumbha",
            "meena",
        ];
        for domain in &domains {
            if lower.contains(domain) {
                return Some(Intent::Wheel {
                    domain: Some(domain.to_string()),
                });
            }
        }
        return Some(Intent::Wheel { domain: None });
    }
    None
}

/// "reason from acceleration to distance" / "derive X from Y"
fn parse_reason(lower: &str, original: &str) -> Option<Intent> {
    if lower.starts_with("reason from ") || lower.starts_with("derive ") {
        let prefix_len = if lower.starts_with("reason from ") {
            "reason from ".len()
        } else {
            "derive ".len()
        };
        let rest = original.trim().get(prefix_len..).unwrap_or("");
        // "X to Y" / "X into Y"
        if let Some(pos) = rest.to_lowercase().find(" to ") {
            let have = rest[..pos].trim().to_string();
            let want = rest[pos + 4..].trim().to_string();
            if !have.is_empty() && !want.is_empty() {
                return Some(Intent::Reason {
                    have,
                    want,
                    max_depth: 5,
                });
            }
        }
        if let Some(pos) = rest.to_lowercase().find(" into ") {
            let have = rest[..pos].trim().to_string();
            let want = rest[pos + 6..].trim().to_string();
            if !have.is_empty() && !want.is_empty() {
                return Some(Intent::Reason {
                    have,
                    want,
                    max_depth: 5,
                });
            }
        }
    }
    None
}

/// "process through shikai" / "shikai this query"
fn parse_shikai(lower: &str, original: &str) -> Option<Intent> {
    if lower.starts_with("shikai ") || lower.starts_with("process through shikai ") {
        let query = original
            .trim()
            .trim_start_matches("shikai ")
            .trim_start_matches("process through shikai ")
            .trim()
            .to_string();
        if !query.is_empty() {
            return Some(Intent::Shikai { query });
        }
    }
    None
}

/// "full bankai solve for X" / "bankai X"
fn parse_bankai(lower: &str, original: &str) -> Option<Intent> {
    if lower.starts_with("bankai ") || lower.starts_with("full bankai solve ") {
        let query = original
            .trim()
            .trim_start_matches("bankai ")
            .trim_start_matches("full bankai solve ")
            .trim_start_matches("for ")
            .trim()
            .to_string();
        if !query.is_empty() {
            return Some(Intent::BankaiSolve { query });
        }
    }
    None
}

// ── System action parsers ──────────────────────────────────────

fn parse_timer(lower: &str) -> Option<Intent> {
    if !lower.contains("timer") && !lower.contains("set a") {
        return None;
    }

    if lower.contains("cancel") {
        let label = extract_label(lower, &["cancel", "timer"]);
        return Some(Intent::CancelTimer { label });
    }

    let parts: Vec<&str> = lower.split_whitespace().collect();
    let mut duration_secs: u64 = 0;
    let mut label = None;

    for (i, word) in parts.iter().enumerate() {
        if let Ok(n) = word.parse::<u64>() {
            let next = parts.get(i + 1).copied().unwrap_or("");
            match next {
                "second" | "seconds" | "sec" | "secs" => duration_secs = n,
                "minute" | "minutes" | "min" | "mins" => duration_secs = n * 60,
                "hour" | "hours" | "hr" | "hrs" => duration_secs = n * 3600,
                _ => duration_secs = n * 60,
            }
        }
    }

    if let Some(for_pos) = lower.find(" for ") {
        let after = &lower[for_pos + 5..];
        let label_text = after.split_whitespace().collect::<Vec<&str>>().join(" ");
        if !label_text.is_empty() {
            label = Some(label_text);
        }
    }

    if duration_secs > 0 {
        Some(Intent::SetTimer {
            duration_secs,
            label,
        })
    } else {
        None
    }
}

fn parse_alarm(lower: &str) -> Option<Intent> {
    if !lower.contains("alarm") {
        return None;
    }
    let time = extract_time(lower).unwrap_or_else(|| "unknown".to_string());
    Some(Intent::SetAlarm { time })
}

fn parse_reminder(lower: &str) -> Option<Intent> {
    if !lower.contains("remind") && !lower.contains("reminder") {
        return None;
    }
    let when = extract_time(lower).unwrap_or_else(|| "later".to_string());
    let text = lower
        .trim_start_matches("remind me to")
        .trim_start_matches("remind me")
        .trim_start_matches("reminder to")
        .trim_start_matches("reminder:")
        .trim()
        .to_string();

    if text.is_empty() {
        return Some(Intent::SetReminder {
            text: "something".to_string(),
            when,
        });
    }
    Some(Intent::SetReminder { text, when })
}

#[cfg(feature = "termux")]
fn parse_sms(lower: &str, original: &str) -> Option<Intent> {
    if !lower.starts_with("text ") && !lower.starts_with("send ") && !lower.starts_with("message ")
    {
        return None;
    }

    let after = if lower.starts_with("send a message to ") {
        original.trim_start_matches("send a message to ")
    } else if lower.starts_with("send message to ") {
        original.trim_start_matches("send message to ")
    } else if lower.starts_with("text ") {
        original.trim_start_matches("text ")
    } else if lower.starts_with("message ") {
        original.trim_start_matches("message ")
    } else {
        return None;
    };

    let mut parts = after.splitn(2, ' ');
    let contact = parts.next()?.trim().to_string();
    let message = parts.next().unwrap_or("").trim().to_string();

    if contact.is_empty() {
        return None;
    }
    Some(Intent::SendMessage { contact, message })
}

#[cfg(feature = "termux")]
fn parse_call(lower: &str, original: &str) -> Option<Intent> {
    if !lower.starts_with("call ") {
        return None;
    }
    let contact = original
        .trim()
        .trim_start_matches("call ")
        .trim_start_matches("Call ")
        .trim_start_matches("phone ")
        .trim_start_matches("Phone ")
        .trim()
        .to_string();
    if contact.is_empty() {
        return None;
    }
    Some(Intent::Call { contact })
}

// ── Helpers ────────────────────────────────────────────────────

fn extract_time(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if word.contains("am") || word.contains("pm") || word.contains(':') {
            return Some(word.to_string());
        }
        if (*word == "at" || *word == "for") && i + 1 < words.len() {
            let next = words[i + 1];
            if next
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Some(next.to_string());
            }
        }
    }
    None
}

fn extract_label(text: &str, skip_words: &[&str]) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut collecting = false;
    let mut label: Vec<&str> = Vec::new();

    for word in &words {
        if skip_words.contains(word) {
            collecting = true;
            continue;
        }
        if collecting && !word.chars().all(|c| c.is_ascii_digit()) {
            label.push(word);
        }
    }

    if label.is_empty() {
        None
    } else {
        Some(label.join(" "))
    }
}

/// Parse "X against Y" → (X, Y)
fn parse_against(text: &str) -> Option<(String, String)> {
    if let Some(pos) = text.to_lowercase().find(" against ") {
        let left = text[..pos].trim().to_string();
        let right = text[pos + 9..].trim().to_string();
        if !left.is_empty() && !right.is_empty() {
            return Some((left, right));
        }
    }
    None
}

/// Heuristic: does this text look like a math expression?
fn looks_like_math(text: &str) -> bool {
    let has_digits = text.chars().any(|c| c.is_ascii_digit());
    let has_operators = text.chars().any(|c| "+-*/^%()".contains(c));
    let has_math_fn = text.contains("sqrt")
        || text.contains("sin")
        || text.contains("cos")
        || text.contains("tan")
        || text.contains("log")
        || text.contains("ln")
        || text.contains("exp")
        || text.contains("abs")
        || text.contains("pow");
    (has_digits && has_operators) || has_math_fn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_basic() {
        let intent = classify("set a 5 minute timer");
        match intent {
            Intent::SetTimer { duration_secs, .. } => assert_eq!(duration_secs, 300),
            _ => panic!("expected SetTimer"),
        }
    }

    #[test]
    fn timer_with_label() {
        let intent = classify("set a 10 minute timer for pasta");
        match intent {
            Intent::SetTimer {
                duration_secs,
                label,
            } => {
                assert_eq!(duration_secs, 600);
                assert_eq!(label.as_deref(), Some("pasta"));
            }
            _ => panic!("expected SetTimer"),
        }
    }

    #[test]
    fn sms_basic() {
        let intent = classify("text John hello");
        match intent {
            Intent::SendMessage { contact, message } => {
                assert_eq!(contact, "John");
                assert_eq!(message, "hello");
            }
            _ => panic!("expected SendMessage"),
        }
    }

    #[test]
    fn call_basic() {
        let intent = classify("call Sarah");
        assert_eq!(
            intent,
            Intent::Call {
                contact: "Sarah".into()
            }
        );
    }

    #[test]
    fn battery() {
        assert_eq!(classify("what's my battery"), Intent::BatteryStatus);
    }

    #[test]
    fn goodbye() {
        assert_eq!(classify("goodbye"), Intent::Goodbye);
    }

    #[test]
    fn help() {
        assert_eq!(classify("help"), Intent::Help);
    }

    #[test]
    fn query() {
        match classify("what is the capital of France") {
            Intent::Query { text } => assert!(text.contains("capital")),
            _ => panic!("expected Query"),
        }
    }

    // ── Ecosystem tests ──────────────────────────────────────

    #[test]
    fn convert_basic() {
        match classify("convert 100 miles to kilometers") {
            Intent::Convert { value, from, to } => {
                assert_eq!(value, 100.0);
                assert_eq!(from, "miles");
                assert_eq!(to, "kilometers");
            }
            _ => panic!("expected Convert"),
        }
    }

    #[test]
    fn convert_no_number() {
        match classify("convert miles to kilometers") {
            Intent::Convert { value, from, to } => {
                assert_eq!(value, 1.0);
                assert_eq!(from, "miles");
                assert_eq!(to, "kilometers");
            }
            _ => panic!("expected Convert"),
        }
    }

    #[test]
    fn eval_basic() {
        match classify("calculate 2+2") {
            Intent::Eval { expression } => assert!(expression.contains("2+2")),
            _ => panic!("expected Eval"),
        }
    }

    #[test]
    fn eval_sqrt() {
        match classify("what's sqrt(144)") {
            Intent::Eval { expression } => assert!(expression.contains("sqrt")),
            _ => panic!("expected Eval"),
        }
    }

    #[test]
    fn validate_basic() {
        match classify("validate this claim: water is H2O") {
            Intent::Validate { text, .. } => assert!(text.contains("water")),
            _ => panic!("expected Validate"),
        }
    }

    #[test]
    fn validate_against() {
        match classify("validate X against Y") {
            Intent::Validate { text, context } => {
                assert_eq!(text, "x");
                assert_eq!(context, "y");
            }
            _ => panic!("expected Validate"),
        }
    }

    #[test]
    fn score_basic() {
        match classify("score this response") {
            Intent::Score { text } => assert!(text.contains("score")),
            _ => panic!("expected Score"),
        }
    }

    #[test]
    fn solve_basic() {
        match classify("prove that 2+2=4") {
            Intent::Solve { query } => assert!(query.contains("2+2=4")),
            _ => panic!("expected Solve"),
        }
    }

    #[test]
    fn formula_basic() {
        match classify("evaluate the circle_area formula with r=5") {
            Intent::Formula { name, .. } => assert_eq!(name, "circle_area"),
            _ => panic!("expected Formula"),
        }
    }

    #[test]
    fn search_formulas_basic() {
        match classify("search formulas for gravity") {
            Intent::SearchFormulas { keyword } => assert_eq!(keyword, "gravity"),
            _ => panic!("expected SearchFormulas"),
        }
    }

    #[test]
    fn traverse_basic() {
        match classify("traverse the mangala domain") {
            Intent::Traverse { domain, .. } => assert_eq!(domain, "mangala"),
            _ => panic!("expected Traverse"),
        }
    }

    #[test]
    fn classify_token() {
        match classify("classify mercury") {
            Intent::Classify { token } => assert_eq!(token, "mercury"),
            _ => panic!("expected Classify"),
        }
    }

    #[test]
    fn wheel_basic() {
        match classify("show the wheel") {
            Intent::Wheel { .. } => {}
            _ => panic!("expected Wheel"),
        }
    }

    #[test]
    fn wheel_with_domain() {
        match classify("show the aries wheel") {
            Intent::Wheel { domain } => assert_eq!(domain.as_deref(), Some("aries")),
            _ => panic!("expected Wheel"),
        }
    }

    #[test]
    fn reason_basic() {
        match classify("reason from acceleration to distance") {
            Intent::Reason { have, want, .. } => {
                assert_eq!(have, "acceleration");
                assert_eq!(want, "distance");
            }
            _ => panic!("expected Reason"),
        }
    }

    #[test]
    fn shikai_basic() {
        match classify("shikai what is gravity") {
            Intent::Shikai { query } => assert!(query.contains("gravity")),
            _ => panic!("expected Shikai"),
        }
    }

    #[test]
    fn bankai_basic() {
        match classify("bankai solve for the meaning of life") {
            Intent::BankaiSolve { query } => assert!(query.contains("meaning of life")),
            _ => panic!("expected BankaiSolve"),
        }
    }
}
