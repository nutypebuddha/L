use super::GateValidator;
use crate::core::ball::{Ball, GateResult};
use crate::core::pin::Gate;
use crate::kb::facts::KnowledgeBase;

pub struct FactGate<'a> {
    kb: &'a KnowledgeBase,
    claim: String,
}

impl<'a> FactGate<'a> {
    pub fn new(kb: &'a KnowledgeBase) -> Self {
        FactGate {
            kb,
            claim: String::new(),
        }
    }

    /// Attach the full claim text so consistency checks can see the whole
    /// statement (e.g. compare a number against an entity mentioned elsewhere
    /// in the sentence), not just the current candidate token.
    pub fn with_claim(mut self, claim: &str) -> Self {
        self.claim = claim.to_string();
        self
    }

    fn check_fact_exists(&self, token: &str, context: &str) -> (bool, f64) {
        let lower_token = token.to_ascii_lowercase();
        let lower_context = context.to_ascii_lowercase();

        if let Some(fact) = self.kb.lookup(&lower_token) {
            let source_score = if fact.source.is_empty() { 0.9 } else { 0.95 };
            return (true, source_score);
        }

        for fact in &self.kb.facts {
            if lower_context.contains(&fact.name) || lower_token.contains(&fact.name) {
                return (true, 0.95);
            }
        }

        if token.bytes().all(|c| {
            c.is_ascii_digit() || c == b'.' || c == b'-' || c == b'+' || c == b'e' || c == b'E'
        }) {
            return (true, 0.8);
        }

        if !self.kb.search(&lower_token).is_empty() || !self.kb.search(&lower_context).is_empty() {
            return (true, 0.85);
        }

        let is_factual = token.parse::<f64>().is_ok()
            || token.contains("speed")
            || token.contains("density")
            || token.contains("mass")
            || token.contains("distance")
            || token.contains("temperature")
            || token.contains("pressure")
            || token.contains("energy")
            || token.contains("force")
            || token.contains("volume")
            || token.contains("area");
        if is_factual {
            (false, 0.3)
        } else {
            (true, 0.6)
        }
    }

    fn check_fact_consistent(&self, token: &str, context: &str) -> (bool, f64) {
        let lower_token = token.to_lowercase();
        let lower_context = context.to_lowercase();

        // Tier 2: reject size claims that contradict a known entity size
        // (e.g. "an i32 occupies 8 bytes" when i32 = 4). Returns Some only
        // when a numeric mismatch is found; otherwise falls through.
        if let Some(decision) = self.check_entity_size() {
            return decision;
        }

        if let (Ok(token_val), Ok(context_val)) =
            (lower_token.parse::<f64>(), lower_context.parse::<f64>())
        {
            if context_val != 0.0 {
                let ratio = token_val / context_val;
                if ratio > 0.1 && ratio < 10.0 {
                    return (true, 0.9);
                } else {
                    return (false, 0.3);
                }
            }
        }

        if let (Ok(token_val), Some(fact)) =
            (lower_token.parse::<f64>(), self.kb.lookup(&lower_context))
        {
            if fact.value != 0.0 {
                let ratio = token_val / fact.value;
                if (0.9..=1.1).contains(&ratio) {
                    return (true, 0.95);
                }
            }
        }

        (true, 0.75)
    }

    /// Tier-2 size validation: if the claim asserts an entity's memory size
    /// (e.g. "a Vec is 24 bytes"), compare the stated number against the
    /// knowledge base. Returns `Some((false, 0.1))` on a numeric mismatch,
    /// `None` when there is no size assertion or the number is consistent
    /// (so the caller's normal logic proceeds).
    fn check_entity_size(&self) -> Option<(bool, f64)> {
        let claim_l = self.claim.to_ascii_lowercase();
        let size_kw = [
            "bytes", "byte", "size", "occupies", "overhead", "layout", "word", "words",
        ];
        if !size_kw.iter().any(|k| claim_l.contains(k)) {
            return None;
        }
        // Assertion verbs that bind an entity to a number ("Vec is 24 bytes").
        let assertion = ["is", "are", "occupies", "takes", "equals", "=", "=="];
        let tokens: Vec<&str> = claim_l
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '.')
            .filter(|t| !t.is_empty())
            .collect();

        for i in 0..tokens.len() {
            let fact = self
                .kb
                .facts
                .iter()
                .find(|f| !f.name.is_empty() && f.value != 0.0 && f.name.eq_ignore_ascii_case(tokens[i]));
            let Some(fact) = fact else {
                continue;
            };
            let start = i.saturating_sub(3);
            let end = (i + 4).min(tokens.len());
            for j in start..end {
                if j == i {
                    continue;
                }
                if let Ok(num) = tokens[j].parse::<f64>() {
                    let lo = j.min(i);
                    let hi = j.max(i);
                    let has_verb = tokens[lo..=hi]
                        .iter()
                        .any(|&t| assertion.contains(&t));
                    if has_verb {
                        let ratio = num / fact.value;
                        if !(0.9..=1.1).contains(&ratio) {
                            return Some((false, 0.1));
                        }
                    }
                }
            }
        }
        None
    }

    fn check_source_reliable(&self, _token: &str, context: &str) -> (bool, f64) {
        let reliable_sources = [
            "wikipedia",
            "nasa",
            "nih",
            "nist",
            "ieee",
            "acm",
            "nature",
            "science",
        ];
        let lower_context = context.to_lowercase();

        for source in &reliable_sources {
            if lower_context.contains(source) {
                return (true, 0.95);
            }
        }

        if context.is_empty() {
            return (true, 0.7);
        }

        (true, 0.8)
    }

    fn check_no_contradiction(&self, token: &str, context: &str) -> (bool, f64) {
        let lower_token = token.to_lowercase();
        let lower_context = context.to_lowercase();

        let fact_contradictions = [
            ("sun is cold", "sun is hot"),
            ("earth is flat", "earth is round"),
            ("water is dry", "water is wet"),
            ("light is slow", "light is fast"),
        ];

        for (false_fact, true_fact) in &fact_contradictions {
            if lower_context.contains(false_fact) && lower_token.contains(true_fact) {
                return (false, 0.1);
            }
            if lower_context.contains(true_fact) && lower_token.contains(false_fact) {
                return (false, 0.1);
            }
            // Standalone false claim (e.g. "the earth is flat") — the whole
            // claim is available via self.claim regardless of tokenization.
            if self.claim.to_ascii_lowercase().contains(false_fact) {
                return (false, 0.1);
            }
        }

        (true, 0.85)
    }
}

impl<'a> GateValidator for FactGate<'a> {
    fn validate(&self, ball: &mut Ball, context: &str) -> GateResult {
        let token = &ball.candidate.token;

        let (exists_ok, exists_score) = self.check_fact_exists(token, context);
        let (consistent_ok, consistent_score) = self.check_fact_consistent(token, context);
        let (reliable_ok, reliable_score) = self.check_source_reliable(token, context);
        let (no_contradiction, contradiction_score) = self.check_no_contradiction(token, context);

        let scores = [
            exists_score,
            consistent_score,
            reliable_score,
            contradiction_score,
        ];
        let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;

        let passed = exists_ok && consistent_ok && reliable_ok && no_contradiction;
        let reason = if !exists_ok {
            Some("Fact not found in knowledge base".to_string())
        } else if !consistent_ok {
            Some("Fact inconsistent with context".to_string())
        } else if !reliable_ok {
            Some("Source reliability uncertain".to_string())
        } else if !no_contradiction {
            Some("Fact contradicts known information".to_string())
        } else {
            None
        };

        if passed {
            GateResult::passed(Gate::Fact, avg_score)
        } else {
            GateResult::failed(Gate::Fact, avg_score, &reason.unwrap_or_default())
        }
    }
}
