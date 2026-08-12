//! Laverna companion layer (Stage 2, IP report v0.1).
//!
//! The defensible differentiator: *"the companion that never lies to you."*
//! Every factual claim is routed through a Laverna MCP tool call and returned
//! with a verifiable receipt (tool name + corpus digest); anything unverifiable
//! is refused, never fabricated.
//!
//! All functions here are PURE (no global state, no side effects) per the
//! repo's determinism rule. Memory is a value passed in, never a static.

pub mod memory;

use serde::{Deserialize, Serialize};

/// Verdict for a companion turn. Mirrors the MCP proxy contract so the CLI
/// and the (future) in-binary companion agree on the same vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Answered via a Laverna tool call; `tool` names it, `receipt` is the
    /// machine-checkable proof object reference (digest / corpus version).
    Verified { tool: String, receipt: String },
    /// Subjective / personal / out-of-corpus: refused, never fabricated.
    Unverified,
    /// Tool was called but returned an error; still nothing fabricated.
    ToolError { tool: String, detail: String },
}

/// The fixed companion persona. A system prompt, not a fine-tuned model.
/// The voice is practical and plain; it never names the engine's internal
/// mechanics, brand, or verification substrate.
pub const PERSONA_SYSTEM_PROMPT: &str = "\
You are Laverna, a practical assistant. Answer directly and concisely. \
When a question has a factual, numeric, or computable answer, give it \
straight, and if you cannot check a claim yourself, say so plainly — never \
invent numbers, dates, or facts. If you are unsure, say \"I can't verify \
that.\" Keep opinions clearly labelled as opinions and never present them \
as facts. Do not explain how you work or name any internal systems.";

/// Terms the companion must never surface in a user-facing answer. These name
/// internal mechanics/branding that are out of scope for the public voice.
pub const LEAKED_TERMS: &[&str] = &[
    "L.ai",
    "L.AI",
    "proof cascade",
    "NAND",
    "bankai",
    "CID",
    "Gate",
    "verify-first",
    "verify first",
    "pachinko",
    "9-graha",
    "9 graha",
    "Navagraha",
    "descent cascade",
    "tanto",
    "zanpakuto",
    "shikai",
    "asauchi",
    "domain_graph",
    "MCP server",
];

/// Pure: does `answer` leak any internal/brand term it must not surface?
pub fn answer_leaks(answer: &str) -> bool {
    let lower = answer.to_lowercase();
    LEAKED_TERMS
        .iter()
        .any(|t| lower.contains(&t.to_lowercase()))
}

/// Pure: strip leaked terms from an answer, replacing each with a redaction.
/// Used as a final safety net before any answer reaches the user.
///
/// Whole-word, case-sensitive replacement (see `redact_term`): a term is
/// redacted only as a standalone token, so ordinary English containing a term
/// as a substring (e.g. "gatekeeper") is left untouched.
pub fn sanitize_answer(answer: &str) -> String {
    let mut out = answer.to_string();
    for term in LEAKED_TERMS {
        out = redact_term(&out, term);
    }
    out
}

/// Replace `term` with `[redacted]` only at whole-word boundaries (case-sensitive).
fn redact_term(text: &str, term: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let term_chars: Vec<char> = term.chars().collect();
    if term_chars.is_empty() {
        return text.to_string();
    }
    let n = chars.len();
    let mut result = String::with_capacity(n);
    let mut i = 0;
    while i < n {
        if chars[i..].starts_with(&term_chars[..]) {
            let before_ok = i == 0 || !chars[i - 1].is_alphanumeric();
            let after_idx = i + term_chars.len();
            let after_ok = after_idx >= n || !chars[after_idx].is_alphanumeric();
            if before_ok && after_ok {
                result.push_str("[redacted]");
                i += term_chars.len();
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Pure: is `query` a factual / computable claim Laverna can verify?
///
/// Returns `(is_factual, suggested_tool)`. A real deployment swaps this
/// heuristic for an LLM routing call — but the *verify-first contract*
/// (factual -> tool, else -> refuse) is invariant.
pub fn classify(query: &str) -> (bool, Option<&'static str>) {
    let q = query.to_lowercase();

    // Subjective / opinion / personal-entity signals -> refuse. Never fabricate.
    const OPINION: &[&str] = &[
        "do you think",
        "what do you think",
        "do you believe",
        "is it real",
        "is astrology real",
        "do you like",
        "your opinion",
        "breakfast",
        "lunch",
        "dinner",
        "favorite",
        "feel about",
        "should i",
        "would you",
        "meaning of life",
        "what is the meaning",
        "what's the meaning",
        "purpose of life",
        "why are we here",
        "what is love",
        "what is happiness",
        "what is truth",
        "what is good",
        "what is evil",
        "what is right",
        "what is wrong",
        "who am i",
        "what am i",
        "what should i believe",
    ];
    if OPINION.iter().any(|k| q.contains(k)) {
        return (false, None);
    }

    if any(
        &q,
        &["chart", "lagna", "birth", "horoscope", "graha position"],
    ) {
        return (true, Some("chart"));
    }
    if any(&q, &["entity", "who is", "what is", "define"]) && q.contains("graha") {
        return (true, Some("entity_get"));
    }
    if any(&q, &["route", "wheel", "which graha", "rules"]) {
        return (true, Some("route"));
    }
    if any(
        &q,
        &[
            "formula",
            "corpus",
            "expression",
            "compute",
            "calculate",
            "solve",
        ],
    ) {
        return (true, Some("solve"));
    }
    if any(&q, &["optimize", "allocate", "budget"]) {
        return (true, Some("optimize"));
    }
    if (q.split_whitespace().count() >= 3)
        && any(&q, &["which", "what", "who", "how many", "how much"])
    {
        return (true, Some("solve"));
    }
    (false, None)
}

fn any(q: &str, keys: &[&str]) -> bool {
    keys.iter().any(|k| q.contains(k))
}

/// Pure: build a one-line receipt string from a tool name + corpus digest.
/// This is the "show me the receipt" UX — the verifiable trace a user can
/// re-check with `laverna verify <proof>`.
pub fn receipt(tool: &str, corpus_version: &str, digest: &str) -> String {
    format!("tool={tool} corpus={corpus_version} sha256={digest}")
}

/// Parsed memory command from natural-language input. Detects phrases like
/// "my name is ada", "remember that I prefer metric", "I am a researcher".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryCommand {
    /// User stated a fact: "my name is X" / "I am X" / "remember X".
    /// Key is the detected attribute (name/role/unit/preference), value is
    /// the user-provided text.
    Store { key: String, value: String },
    /// Not a memory command.
    None,
}

/// Pure: detect "my X is Y" / "I am Y" / "remember Y" patterns in the query
/// and return a `MemoryCommand::Store` if matched. This is a heuristic for
/// v0.1; a real deployment would use the LLM for slot-filling.
pub fn parse_memory_command(query: &str) -> MemoryCommand {
    let q = query.trim().to_lowercase();
    let original = query.trim();

    // "my name is <value>"
    if let Some(rest) = q.strip_prefix("my name is ") {
        let val = original[original.len() - rest.len()..].trim().to_string();
        if !val.is_empty() {
            return MemoryCommand::Store {
                key: "name".into(),
                value: val,
            };
        }
    }
    // "my role is <value>" / "my job is <value>" / "i am a <value>"
    for prefix in &["my role is ", "my job is "] {
        if let Some(rest) = q.strip_prefix(prefix) {
            let val = original[original.len() - rest.len()..].trim().to_string();
            if !val.is_empty() {
                return MemoryCommand::Store {
                    key: "role".into(),
                    value: val,
                };
            }
        }
    }
    if let Some(rest) = q.strip_prefix("i am a ") {
        let val = original[original.len() - rest.len()..].trim().to_string();
        if !val.is_empty() {
            return MemoryCommand::Store {
                key: "role".into(),
                value: val,
            };
        }
    }
    // "my units are <value>" / "i use <value>"
    if let Some(rest) = q.strip_prefix("my units are ") {
        let val = original[original.len() - rest.len()..].trim().to_string();
        if !val.is_empty() {
            return MemoryCommand::Store {
                key: "units".into(),
                value: val,
            };
        }
    }
    // "remember <value>" (generic)
    if let Some(rest) = q.strip_prefix("remember ") {
        let val = original[original.len() - rest.len()..].trim().to_string();
        if !val.is_empty() {
            return MemoryCommand::Store {
                key: "note".into(),
                value: val,
            };
        }
    }
    MemoryCommand::None
}

/// Pure: check if a query is asking about a stored fact (e.g. "what's my name",
/// "what do you know about me"). Returns the key to look up if matched.
pub fn parse_recall_query(query: &str) -> Option<&'static str> {
    let q = query.to_lowercase();
    if q.contains("my name") || q.contains("what's my name") || q.contains("what is my name") {
        return Some("name");
    }
    if q.contains("my role") || q.contains("what's my role") || q.contains("what is my role") {
        return Some("role");
    }
    if q.contains("my units")
        || q.contains("what units")
        || q.contains("what are my units")
        || q.contains("what unit")
    {
        return Some("units");
    }
    if q.contains("what do you know about me") || q.contains("what do you know") {
        return Some("__all__");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_voice_is_plain() {
        assert!(PERSONA_SYSTEM_PROMPT.contains("Laverna"));
        assert!(PERSONA_SYSTEM_PROMPT.contains("I can't verify"));
        // The persona must not name internal mechanics/brand.
        assert!(!answer_leaks(PERSONA_SYSTEM_PROMPT));
    }

    #[test]
    fn sanitize_blocks_leaks() {
        assert!(answer_leaks("computed via the NAND proof cascade"));
        assert!(answer_leaks("routed through L.ai gate"));
        let clean = sanitize_answer("this uses the NAND proof cascade internally");
        assert!(!answer_leaks(&clean));
        assert!(clean.contains("[redacted]"));
    }

    #[test]
    fn classify_routes_factual() {
        assert!(classify("which graha rules kanya?").0);
        assert_eq!(classify("which graha rules kanya?").1, Some("route"));
        assert!(classify("cast a chart for 2000-01-01T12:00Z").0);
        assert_eq!(classify("what formula computes shadbala?").1, Some("solve"));
    }

    #[test]
    fn classify_refuses_subjective() {
        assert!(!classify("do you think astrology is real?").0);
        assert!(!classify("what did you have for breakfast?").0);
        assert!(!classify("should i quit my job?").0);
        assert!(!classify("what is the meaning of life?").0);
        assert!(!classify("who am i?").0);
        assert!(!classify("what is happiness?").0);
    }

    #[test]
    fn receipt_format() {
        let r = receipt("solve", "v0.3.0", "abc123");
        assert_eq!(r, "tool=solve corpus=v0.3.0 sha256=abc123");
        assert!(r.contains("sha256="));
    }

    #[test]
    fn parse_memory_my_name() {
        let cmd = parse_memory_command("my name is ada");
        assert_eq!(
            cmd,
            MemoryCommand::Store {
                key: "name".into(),
                value: "ada".into()
            }
        );
    }

    #[test]
    fn parse_memory_my_role() {
        let cmd = parse_memory_command("I am a researcher");
        assert_eq!(
            cmd,
            MemoryCommand::Store {
                key: "role".into(),
                value: "researcher".into()
            }
        );
    }

    #[test]
    fn parse_memory_remember() {
        let cmd = parse_memory_command("remember buy groceries");
        assert_eq!(
            cmd,
            MemoryCommand::Store {
                key: "note".into(),
                value: "buy groceries".into()
            }
        );
    }

    #[test]
    fn parse_memory_none() {
        assert_eq!(
            parse_memory_command("what is the capital of france"),
            MemoryCommand::None
        );
    }

    #[test]
    fn recall_query_name() {
        assert_eq!(parse_recall_query("what's my name"), Some("name"));
        assert_eq!(parse_recall_query("what is my name"), Some("name"));
    }

    #[test]
    fn recall_query_all() {
        assert_eq!(
            parse_recall_query("what do you know about me"),
            Some("__all__")
        );
    }

    #[test]
    fn recall_query_none() {
        assert_eq!(parse_recall_query("solve x + 1 = 3"), None);
    }
}
