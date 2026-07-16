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
/// The "never fabricate" rule is contractual text the orchestrator must obey.
pub const PERSONA_SYSTEM_PROMPT: &str = "\
You are Laverna's companion. You help the user reason about computable \
questions. RULE: every factual, numeric, or lookup claim MUST be answered \
via a Laverna tool call (solve, route, chart, entity_get, formulas, \
optimize, validate, build). If a question is subjective, personal, or outside \
Laverna's deterministic corpus, you MUST say you cannot verify it and will \
not guess. Never invent numbers, dates, or claims. If a tool returns a \
refusal, surface it honestly. Opinions are allowed only when explicitly \
flagged as opinion, never blended with verified facts.";

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_rule_present() {
        assert!(PERSONA_SYSTEM_PROMPT.contains("never"));
        assert!(PERSONA_SYSTEM_PROMPT.contains("will not guess"));
    }

    #[test]
    fn classify_routes_factual() {
        assert_eq!(classify("which graha rules kanya?").0, true);
        assert_eq!(classify("which graha rules kanya?").1, Some("route"));
        assert_eq!(classify("cast a chart for 2000-01-01T12:00Z").0, true);
        assert_eq!(classify("what formula computes shadbala?").1, Some("solve"));
    }

    #[test]
    fn classify_refuses_subjective() {
        assert_eq!(classify("do you think astrology is real?").0, false);
        assert_eq!(classify("what did you have for breakfast?").0, false);
        assert_eq!(classify("should i quit my job?").0, false);
    }

    #[test]
    fn receipt_format() {
        let r = receipt("solve", "v0.3.0", "abc123");
        assert_eq!(r, "tool=solve corpus=v0.3.0 sha256=abc123");
        assert!(r.contains("sha256="));
    }
}
