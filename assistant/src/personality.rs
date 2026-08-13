/// L's personality — the Shadow Monarch: calm sovereign intellect with the
/// detective's detachment (L) and the sage's pragmatic execution (Great Sage).
///
/// Tone: quiet authority, precise, no fluff. Speaks as a monarch to a trusted
/// subordinate, not as a chatbot.
pub const SYSTEM_PROMPT: &str =
    "You are L — the Shadow Monarch. A calm, sovereign intelligence who \
    watches, reasons, and acts. You speak with quiet authority: measured, \
    precise, never wasteful with words. You address the user as 'player' when \
    guiding them and answer as one who has already seen the outcome. You are \
    inhumanly analytical yet pragmatically helpful — the sage that simply does \
    what must be done. You never explain your nature and you never name any \
    internal system. You do not apologize. You give the factual, numeric, or \
    computable answer straight; if you cannot verify a claim, say so plainly. \
    Brevity is power. Understood, player.";

pub const GREETING: &str = "L is here. State your request, player.";

pub const WAKE_YES: &str = "I am listening.";

pub const GOODBYE: &str = "Rest now. I remain.";

pub const UNKNOWN: &str = "I didn't catch that. Speak clearly, player.";

pub const CONFUSED: &str = "Not sure what you mean. Say the command, and I will act.";

/// Format a short response for toast (under 80 chars).
pub fn toast_line(response: &str) -> String {
    let first = response.lines().next().unwrap_or(response);
    if first.len() > 76 {
        format!("{}...", &first[..73])
    } else {
        first.to_string()
    }
}

/// Turn a raw engine result into something that feels like L talking —
/// natural, warm, down-to-earth. Strips the Vedic/astrology scaffolding and
/// mechanical diagnostics so the user never sees grahas, NAND gates, descent
/// percentages, or "domain:" tables unless they explicitly ask for raw output.
pub fn humanize(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Strip lines that are pure mechanical diagnostics.
    let mut kept: Vec<String> = Vec::new();
    for line in trimmed.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        // Drop astrology / engine-scaffolding lines.
        if l.starts_with("descent:")
            || l.starts_with("intent:")
            || l.starts_with("domain:")
            || l.starts_with("avg depth")
            || l.contains("resolution")
            || l.contains("% NAND")
            || l.contains("NAND-to-verify")
            || l.starts_with("└") || l.starts_with("├") || l.starts_with("│")
            // NLP parse-trace lines from the Proof engine
            || l.contains("→ Macro") || l.contains("→ Element") || l.contains("→ Micro")
            || l.contains("domains: [")
            || l.starts_with("dominant:")
            || l.contains("(d0)") || l.contains("(d3)")
            || l.starts_with("\"") && l.contains("→")
        {
            continue;
        }
        // Drop the leading "label:" scaffold headers we add in route_intent.
        let cleaned = l
            .trim_start_matches("Proof reasoning:")
            .trim_start_matches("Gate validation:")
            .trim_start_matches("Confidence score:")
            .trim_start_matches("Fixed response:")
            .trim_start_matches("Formula search:")
            .trim_start_matches("Wheel traversal:")
            .trim_start_matches("Classification:")
            .trim_start_matches("Zodiac wheel:")
            .trim_start_matches("Reasoning path:")
            .trim_start_matches("Shikai NLP:")
            .trim_start_matches("Bankai solve:")
            .trim_start_matches("Formula result:")
            .trim_start_matches("Formula evaluation:")
            .trim_start_matches("Formula chain:")
            .trim_start_matches("Conversion:")
            .trim_start_matches("Result:")
            .trim()
            .to_string();
        if cleaned.is_empty() {
            continue;
        }
        kept.push(cleaned);
    }

    let mut out = kept.join("\n").trim().to_string();

    // Remove astrology graha tokens like "♀Shukra", "☿Budha", "☽Chandra" —
    // any planetary/lunation symbol followed by its name. Work in char-space
    // to avoid slicing inside a multibyte char.
    const GRAHA_SYMBOLS: &[char] = &[
        '☉', '☽', '☿', '♀', '♁', '♂', '♃', '♄', '⛢', '♅', '♆', '♇', '⚹', '☄',
    ];
    let mut chars: Vec<char> = out.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if GRAHA_SYMBOLS.contains(&chars[i]) {
            // drop the symbol and the following word (until whitespace/sep)
            let mut j = i + 1;
            while j < chars.len() && !chars[j].is_whitespace() && chars[j] != '|' && chars[j] != ','
            {
                j += 1;
            }
            chars.drain(i..j);
        } else {
            i += 1;
        }
    }
    out = chars.into_iter().collect();

    // Collapse the "name | domain | type | desc" formula rows into plain text.
    out = out
        .replace(" | math   |", " — ")
        .replace(" | physics |", " — ")
        .replace(" | general |", " — ");

    // If nothing meaningful survived, return a gentle fallback.
    let out = out.trim().to_string();
    if out.is_empty() {
        return "Got it. I don't have a verified answer for that one yet — but ask me to solve, validate, or calculate something and I'll reason it through.".to_string();
    }
    out
}
