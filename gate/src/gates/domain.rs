// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0

use super::GateValidator;
use crate::core::ball::{Ball, GateResult};
use crate::core::pin::Gate;

/// DomainBindingGate: a token is *domain-bound* iff every non-trivial content
/// word it contains is grounded in the verified context it is allowed to draw
/// from. A word is grounded when it appears (case-insensitively) in that
/// context.
///
/// This is the "reject any token lacking domain binding" gate: a generated
/// token whose vocabulary cannot be traced back to the verified source has no
/// provenance and is refused, eliminating off-domain or fabricated output.
pub struct DomainBindingGate;

impl DomainBindingGate {
    pub fn new() -> Self {
        DomainBindingGate
    }

    /// Content words below this length are skipped (articles, pronouns, short
    /// conjunctions carry no domain weight).
    const MIN_WORD_LEN: usize = 3;

    fn is_stopword(word: &str) -> bool {
        const STOP: &[&str] = &[
            "the", "and", "for", "are", "but", "not", "you", "all", "any", "can", "has", "was",
            "were", "will", "with", "that", "this", "from", "into", "than", "then", "they", "them",
            "his", "her", "its", "our", "your", "their", "there", "here", "what", "when", "where",
            "which", "who", "why", "how", "also", "may", "might", "must", "shall", "should",
            "would", "could", "does", "did", "each", "more", "most", "some", "such", "only",
            "over", "under", "between", "within", "without",
        ];
        STOP.contains(&word)
    }

    /// Split `text` into lowercase alnum content words (stopwords & short words
    /// already filtered out) that must be present in the context.
    fn content_words(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter_map(|w| {
                let w = w.trim().to_lowercase();
                if w.len() >= Self::MIN_WORD_LEN && !Self::is_stopword(&w) {
                    Some(w)
                } else {
                    None
                }
            })
            .collect()
    }

    /// True when `word` occurs as a substring of `context` (case-insensitive).
    fn grounded(word: &str, context_lower: &str) -> bool {
        context_lower.contains(word)
    }
}

impl Default for DomainBindingGate {
    fn default() -> Self {
        Self::new()
    }
}

impl GateValidator for DomainBindingGate {
    fn validate(&self, ball: &mut Ball, context: &str) -> GateResult {
        let token = ball.candidate.token.trim();
        let context_lower = context.to_lowercase();

        // Nothing to bind against: prove it later, do not emit a fabricated Pass.
        if context_lower.is_empty() {
            return GateResult::unevaluable(
                Gate::DomainBinding,
                0.0,
                "no verified grounding context supplied",
            );
        }

        let words = Self::content_words(token);
        if words.is_empty() {
            // Either the token is punctuation/stopwords only, or already an
            // equation term handled by ProofRecompute — nothing to bind.
            return GateResult::unevaluable(
                Gate::DomainBinding,
                0.0,
                "no content words requiring domain binding",
            );
        }

        let ungrounded: Vec<&String> = words
            .iter()
            .filter(|w| !Self::grounded(w, &context_lower))
            .collect();

        if ungrounded.is_empty() {
            GateResult::passed(Gate::DomainBinding, 1.0)
        } else {
            let detail = ungrounded
                .iter()
                .map(|w| w.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            GateResult::failed(
                Gate::DomainBinding,
                0.0,
                &format!("unbound content words (no provenance): {detail}"),
            )
        }
    }
}
