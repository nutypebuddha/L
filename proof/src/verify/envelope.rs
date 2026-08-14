//! Trusted proof envelope.
//!
//! # Trusted Computing Base (TCB) boundary
//!
//! This module is **inside** the trusted core. Everything it does is pure,
//! deterministic, and free of any model, network, or heuristic state:
//!
//! - **Trusted (kept here / in the primitive + verifier):** deterministic
//!   primitives, proof *construction*, proof *verification*, hashing
//!   (`crate::digest`), corpus-integrity references, and *refusal* semantics.
//! - **Untrusted (never inside this module):** LLMs, NLP, Athena, heuristics,
//!   scoring, and external adapters. They may *propose* a claim or supply a
//!   derivation string, but they cannot mint or validate a `ProofEnvelope`.
//!   The Verifier (`verifier.rs`) decides; this envelope only records and
//!   seals that decision.
//!
//! A `ProofEnvelope` is the machine-checkable artifact a verification produces.
//! Its `proof_hash` commits to every field *except* the hash itself, so any
//! tampering is detectable by re-deriving and comparing. The envelope is
//! versioned so that verification semantics remain reproducible across engine
//! releases.

use crate::digest::{sha256_hex, to_hex};
use serde::{Deserialize, Serialize};

/// Version of the *proof envelope format* (distinct from the engine version).
/// Bump when the sealed-payload shape changes so old verifiers reject new
/// envelopes instead of misreading them.
pub const PROOF_ENVELOPE_VERSION: u32 = 1;

/// The Verifier's decision recorded in the envelope.
///
/// This is a *decision*, not a probability. It must never be conflated with a
/// heuristic confidence score or a diagnostic `confidence` number — those are
/// untrusted quality signals, not truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofVerdict {
    /// The claim was demonstrated to hold under the stated assumptions.
    Accepted,
    /// The claim was refuted, or refused (out of scope / unverifiable-to-reject).
    Refused,
    /// The Verifier could not reach a decision either way.
    Unevaluable,
}

impl ProofVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProofVerdict::Accepted => "accepted",
            ProofVerdict::Refused => "refused",
            ProofVerdict::Unevaluable => "unevaluable",
        }
    }
}

/// A stable, versioned, tamper-evident proof object.
///
/// Every field except `proof_hash` is part of the sealed (hashed) payload.
/// `seal()` computes the `proof_hash` integrity digest; `verify_integrity()`
/// re-derives it and returns `false` on any mutation. The `proof_hash` is
/// tamper-evidence, not cryptographic authenticity — it commits to the exact
/// bytes produced by this engine, but confers no digital signature or proof of
/// origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEnvelope {
    /// Envelope/proof-format version (`PROOF_ENVELOPE_VERSION`).
    pub proof_version: u32,
    /// Engine (laverna) version that produced the proof.
    pub engine_version: String,
    /// SHA-256 of the canonical input/claim text.
    pub input_hash: String,
    /// SHA-256 of the corpus the proof was verified against.
    pub corpus_hash: String,
    /// Explicit assumptions the derivation relied upon (must be enumerable so a
    /// reviewer can challenge them; an empty list means "no extra assumptions").
    pub assumptions: Vec<String>,
    /// Human/agent-readable derivation trace (ordered steps). Untrusted prose,
    /// but committed to by `proof_hash` so it cannot be silently swapped.
    pub derivation: String,
    /// The result/answer the proof establishes.
    pub result: String,
    /// The Verifier's decision.
    pub verdict: ProofVerdict,
    /// SHA-256 over the canonical serialization of every field above. Excluded
    /// from its own digest.
    pub proof_hash: String,
}

/// The subset of [`ProofEnvelope`] committed by the integrity hash. Keeping it
/// as a dedicated struct guarantees the canonical byte order is single-sourced.
#[derive(Serialize)]
struct SealedPayload<'a> {
    proof_version: u32,
    engine_version: &'a str,
    input_hash: &'a str,
    corpus_hash: &'a str,
    assumptions: &'a [String],
    derivation: &'a str,
    result: &'a str,
    verdict: ProofVerdict,
}

impl ProofEnvelope {
    /// Canonical, deterministic JSON of the sealed payload. Assumptions are
    /// sorted so equivalent envelopes hash identically regardless of insertion
    /// order (determinism rule: sort by stable key before aggregating).
    fn sealed_json(&self) -> String {
        let mut assumptions = self.assumptions.clone();
        assumptions.sort();
        let payload = SealedPayload {
            proof_version: self.proof_version,
            engine_version: &self.engine_version,
            input_hash: &self.input_hash,
            corpus_hash: &self.corpus_hash,
            assumptions: &assumptions,
            derivation: &self.derivation,
            result: &self.result,
            verdict: self.verdict,
        };
        // `serde_json::to_string` serializes struct fields in declaration order,
        // which is itself stable; combined with sorted assumptions this is a
        // fully deterministic digest input.
        serde_json::to_string(&payload).expect("ProofEnvelope sealed payload must serialize")
    }

    /// Compute and store the `proof_hash` for this envelope in place.
    pub fn seal(&mut self) {
        self.proof_hash = sha256_hex(self.sealed_json().as_bytes());
    }

    /// Build a sealed envelope (computes `proof_hash`).
    pub fn new(
        input: &str,
        corpus_hash: &str,
        assumptions: Vec<String>,
        derivation: String,
        result: String,
        verdict: ProofVerdict,
    ) -> Self {
        let mut env = ProofEnvelope {
            proof_version: PROOF_ENVELOPE_VERSION,
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            input_hash: sha256_hex(input.as_bytes()),
            corpus_hash: corpus_hash.to_string(),
            assumptions,
            derivation,
            result,
            verdict,
            proof_hash: String::new(),
        };
        env.seal();
        env
    }

    /// True iff the stored `proof_hash` matches the re-derived digest of the
    /// sealed payload. This is the single integrity check: any field mutation
    /// (including swapping `derivation`) makes it return `false`.
    pub fn verify_integrity(&self) -> bool {
        if self.proof_hash.is_empty() {
            return false;
        }
        sha256_hex(self.sealed_json().as_bytes()) == self.proof_hash
    }

    /// Short, stable fingerprint for logging/tracing (first 16 hex chars).
    pub fn short_hash(&self) -> String {
        self.proof_hash.chars().take(16).collect()
    }
}

/// Convenience: hash an arbitrary input the same way the envelope does, so a
/// caller can compare `input_hash` against an externally computed value.
pub fn hash_input(input: &str) -> String {
    sha256_hex(input.as_bytes())
}

/// Convenience: hex-encode raw bytes (re-export to avoid leaking `digest`).
pub fn hex(bytes: &[u8]) -> String {
    to_hex(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(verdict: ProofVerdict) -> ProofEnvelope {
        ProofEnvelope::new(
            "2 + 3 = 5",
            "corpus-deadbeef",
            vec![
                "arithmetic closure".to_string(),
                "no division by zero".into(),
            ],
            "add(2,3) = 5".to_string(),
            "5".to_string(),
            verdict,
        )
    }

    #[test]
    fn deterministic_replay_same_input_same_hash() {
        // Replaying the identical claim + context yields a byte-identical hash.
        let a = sample(ProofVerdict::Accepted);
        let b = sample(ProofVerdict::Accepted);
        assert_eq!(a.proof_hash, b.proof_hash);
        assert_eq!(a.short_hash(), b.short_hash());
    }

    #[test]
    fn deterministic_replay_order_independent_assumptions() {
        let a = sample(ProofVerdict::Accepted);
        let b = ProofEnvelope::new(
            "2 + 3 = 5",
            "corpus-deadbeef",
            vec!["no division by zero".into(), "arithmetic closure".into()],
            "add(2,3) = 5".to_string(),
            "5".to_string(),
            ProofVerdict::Accepted,
        );
        // Different assumption insertion order must NOT change the hash.
        assert_eq!(a.proof_hash, b.proof_hash);
    }

    #[test]
    fn serde_roundtrip() {
        let a = sample(ProofVerdict::Accepted);
        let json = serde_json::to_string(&a).unwrap();
        let b: ProofEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(a.proof_hash, b.proof_hash);
        assert_eq!(a.verdict, b.verdict);
        assert_eq!(a.assumptions, b.assumptions);
        assert!(b.verify_integrity());
    }

    #[test]
    fn mutation_invalidates_proof() {
        let mut a = sample(ProofVerdict::Accepted);
        assert!(a.verify_integrity());
        // Tamper with the result without re-sealing.
        a.result = "6".to_string();
        assert!(!a.verify_integrity(), "mutated result must fail integrity");
    }

    #[test]
    fn verdict_swap_invalidates_proof() {
        let mut a = sample(ProofVerdict::Accepted);
        a.verdict = ProofVerdict::Refused;
        assert!(!a.verify_integrity());
    }

    #[test]
    fn derivation_swap_invalidates_proof() {
        let mut a = sample(ProofVerdict::Accepted);
        a.derivation = "add(2,3) = 6 ???".to_string();
        assert!(!a.verify_integrity());
    }

    #[test]
    fn assumptions_swap_invalidates_proof() {
        let mut a = sample(ProofVerdict::Accepted);
        a.assumptions.push("extra hidden assumption".to_string());
        assert!(!a.verify_integrity());
    }

    /// Property test: for many varied inputs, a freshly sealed envelope always
    /// verifies, and a single-character mutation of the result always fails.
    #[test]
    fn property_valid_proofs_verify_tampered_fail() {
        // Tiny deterministic PRNG (xorshift) — no external deps, drives property
        // coverage over varied inputs.
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };

        let corpus = "corpus-fixed";
        for _ in 0..256 {
            let len = (next() % 24) as usize + 1;
            let mut input = String::with_capacity(len);
            for _ in 0..len {
                let c = (b'a' + (next() % 26) as u8) as char;
                input.push(c);
            }
            let env = ProofEnvelope::new(
                &input,
                corpus,
                vec!["assumption-a".into()],
                format!("derive({})", input),
                "result".to_string(),
                ProofVerdict::Accepted,
            );
            assert!(
                env.verify_integrity(),
                "sealed proof must verify for input {:?}",
                input
            );
            // Mutate result and confirm it now fails.
            let mut tampered = env.clone();
            tampered.result.push('x');
            assert!(
                !tampered.verify_integrity(),
                "tampered proof must fail for input {:?}",
                input
            );
        }
    }

    #[test]
    fn verdict_is_not_probability() {
        // Documents the contract: the verdict is a decision, never a score.
        assert_ne!(ProofVerdict::Accepted.as_str(), "0.99");
    }
}
