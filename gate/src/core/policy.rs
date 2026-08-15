//! PCN comparison policies for the tri-state gate (Stage 1).
//!
//! Mirrors the four policies proved in Proof-Carrying Numbers (Solatorio,
//! World Bank DECDG, arXiv:2509.06902): a displayed number is `Verified` only
//! when it matches a structured claim under one of these declared policies.
//! `exact` / `rounded` are what Tanto's math comparison already produces;
//! `alias` (sanctioned scale equivalence, e.g. "K" = 10³) and `tolerance`
//! (value within a band *and* hedged with a qualifier like "about") are the
//! remaining PCN policies, kept here so the contract is complete even before
//! every gate emits them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Exact,
    Rounded,
    Alias,
    Tolerance,
}

impl Policy {
    /// Stable string form. Lowercase, never changes — consumed by renderers
    /// and the JSON contract.
    pub fn as_str(&self) -> &'static str {
        match self {
            Policy::Exact => "exact",
            Policy::Rounded => "rounded",
            Policy::Alias => "alias",
            Policy::Tolerance => "tolerance",
        }
    }
}
