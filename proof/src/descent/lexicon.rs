//! Runtime-loadable keyword → domain lexicon (dynamic strategy coverage).
//!
//! `DOMAIN_KEYWORDS` in `descent::mod` is a `const` compiled into the binary —
//! teaching the router a new topic (e.g. "personal finance") today means a
//! Rust PR and a rebuild. `Lexicon` is the same deterministic, weighted
//! mapping, but loaded from an external TOML file at runtime, so an "alien"
//! domain can be handed to `route`/`strategize` in minutes via a small TOML
//! file instead of a recompile cycle.
//!
//! # Precedence (T110)
//! The built-in `DOMAIN_KEYWORDS` table and the dynamic-entity/forms/events
//! path are tried first, completely unchanged — this module only fills gaps.
//! A `Lexicon` entry is consulted **only** when nothing else resolved the
//! token, and every hit is tagged with a distinct provenance and weight
//! (default 0.6) so it never gets confused downstream with a curated
//! DOMAIN_KEYWORDS hit or with the unrelated 0.3-confidence full-text
//! formula-search fallback. This keeps the audited table authoritative and
//! treats the lexicon strictly as an *additive, clearly-labeled* extension —
//! no invented scalars: the weight is an explicit, documented default, not a
//! derived or hashed value.
//!
//! # Format
//! ```toml
//! [[keyword]]
//! word = "ssi"
//! domain = "budha"       # any Domain::parse()-accepted name/symbol/alias
//! weight = 0.6           # optional, defaults to 0.6, must be in (0.0, 1.0]
//!
//! [[keyword]]
//! word = "able account"  # multi-word phrase — matched as an exact span
//! domain = "budha"
//! ```

use std::collections::HashMap;

use crate::domain_graph::Domain;

/// A single resolved lexicon entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LexiconEntry {
    pub domain: Domain,
    pub weight: f64,
}

/// A runtime-loaded keyword → domain table.
///
/// Two exact-match tiers, both case-insensitive and whitespace-normalized:
/// * single-word entries (`entries`) — matched per token via [`Lexicon::lookup`];
/// * multi-word phrases (`phrases`) — matched as a contiguous token span via
///   [`Lexicon::lookup_phrase`].
///
/// Deliberately the simplest possible structure (exact match, no regex, no
/// stemming) — `nlp::stem_token` already collapses inflected and derivational
/// variants (`resilience`→`resilient`, `durability`→`durable`) before the
/// keyword table is consulted, and a `Lexicon` entry is only looked up when
/// that whole chain missed. The MVP mirrors `domain_for_keyword`'s core
/// guarantee: a keyword/phrase never matches as a bare substring fragment,
/// because there is no substring matching here at all, only exact map lookup
/// (whole-word, or a whole multi-word span).
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    entries: HashMap<String, LexiconEntry>,
    phrases: HashMap<String, LexiconEntry>,
}

fn default_weight() -> f64 {
    0.6
}

#[derive(Debug, serde::Deserialize)]
struct LexiconFile {
    #[serde(default)]
    keyword: Vec<LexiconKeywordRaw>,
}

#[derive(Debug, serde::Deserialize)]
struct LexiconKeywordRaw {
    word: String,
    domain: String,
    #[serde(default = "default_weight")]
    weight: f64,
}

impl Lexicon {
    /// Parse a lexicon from TOML text. Fails loudly (T-series rule: no
    /// silent partial loads) on: empty word, an unparseable domain name, or a
    /// weight outside `(0.0, 1.0]`. A `word` is whitespace-normalized, so
    /// `"able  account"` and `"able account"` are the same key. Single-word
    /// entries go into the per-token `entries` map; multi-word entries go into
    /// the span-matching `phrases` map. A duplicate `word` (by normalized key,
    /// across both tiers) is rejected rather than silently letting the last
    /// one win.
    pub fn load_toml(content: &str) -> Result<Self, String> {
        let parsed: LexiconFile =
            toml::from_str(content).map_err(|e| format!("lexicon TOML parse error: {e}"))?;

        let mut entries = HashMap::new();
        let mut phrases = HashMap::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for kw in parsed.keyword {
            let norm: String = kw
                .word
                .trim()
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if norm.is_empty() {
                return Err("lexicon entry has an empty `word`".to_string());
            }
            let domain = Domain::parse(&kw.domain).ok_or_else(|| {
                format!(
                    "lexicon entry {norm:?} has unrecognized domain {:?}",
                    kw.domain
                )
            })?;
            if !(kw.weight > 0.0 && kw.weight <= 1.0) {
                return Err(format!(
                    "lexicon entry {norm:?} has weight {} — must be in (0.0, 1.0]",
                    kw.weight
                ));
            }
            if !seen.insert(norm.clone()) {
                return Err(format!("lexicon has duplicate entry for word {norm:?}"));
            }
            if norm.split_whitespace().count() == 1 {
                entries.insert(
                    norm,
                    LexiconEntry {
                        domain,
                        weight: kw.weight,
                    },
                );
            } else {
                phrases.insert(
                    norm,
                    LexiconEntry {
                        domain,
                        weight: kw.weight,
                    },
                );
            }
        }
        Ok(Lexicon { entries, phrases })
    }

    /// Whole-word, case-insensitive lookup. Returns `None` for anything not
    /// an exact match — never a substring hit.
    pub fn lookup(&self, token: &str) -> Option<LexiconEntry> {
        self.entries.get(&token.to_lowercase()).copied()
    }

    /// Phrase-level, case-insensitive lookup: matches a contiguous run of
    /// tokens starting at the *beginning* of `tokens` against a registered
    /// multi-word phrase. Tries the longest possible span first (bounded at 8
    /// tokens), so a 3-word phrase wins over a 2-word prefix. Returns the
    /// number of tokens consumed and the entry. Single-word entries live in
    /// [`Lexicon::lookup`]; this only resolves multi-word spans, so it
    /// returns `None` when no phrase starts at this position.
    pub fn lookup_phrase(&self, tokens: &[&str]) -> Option<(usize, LexiconEntry)> {
        if self.phrases.is_empty() || tokens.is_empty() {
            return None;
        }
        let max = tokens.len().min(8);
        for len in (2..=max).rev() {
            let candidate: String = tokens[..len]
                .iter()
                .map(|t| t.to_lowercase())
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(entry) = self.phrases.get(&candidate) {
                return Some((len, *entry));
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.entries.len() + self.phrases.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.phrases.is_empty()
    }
}

/// Self-describing TOML template printed by `lai schema lexicon`. Mirrors the
/// exact format `load_toml` accepts, with the validation rules called out so a
/// user can author a runtime lexicon without reading the source.
pub const LEXICON_SCHEMA_TEMPLATE: &str = r#"# Runtime keyword -> domain lexicon for `lai route --lexicon <file>` / `lai strategize --lexicon <file>`.
#
# Teaches the router new vocabulary at runtime (no recompile). The built-in
# DOMAIN_KEYWORDS table and the dynamic-entity path are tried first and always
# win; a lexicon entry is consulted ONLY when nothing else resolved the token.
# Every hit is case-insensitive, whitespace-normalized, and weighted below the
# curated table, so the lexicon is a strictly additive, clearly-labeled
# extension.
#
# Both single words and multi-word phrases are supported (exact match only —
# no substring / stemming). A phrase matches a contiguous run of tokens, so
# "able account" only fires on that exact adjacent pair, never on "account".

[[keyword]]
word = "ssi"            # single word (or "ssi trust" for a phrase)
domain = "budha"        # any Domain name/symbol/alias (surya, mangala, 2, etc.)
weight = 0.6            # optional, in (0.0, 1.0]; defaults to 0.6

[[keyword]]
word = "able_account"
domain = "budha"

[[keyword]]
word = "personal finance"   # multi-word phrase -> matched as an exact span
domain = "budha"
weight = 0.75
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_valid_lexicon() {
        let toml = r#"
            [[keyword]]
            word = "ssi"
            domain = "budha"
            weight = 0.6

            [[keyword]]
            word = "able_account"
            domain = "budha"
        "#;
        let lex = Lexicon::load_toml(toml).unwrap();
        assert_eq!(lex.len(), 2);
        let ssi = lex.lookup("SSI").unwrap(); // case-insensitive
        assert_eq!(ssi.domain, Domain::Budha);
        assert_eq!(ssi.weight, 0.6);
        // default_weight() applied when omitted
        assert_eq!(lex.lookup("able_account").unwrap().weight, 0.6);
    }

    #[test]
    fn lookup_never_matches_substring() {
        let toml = r#"
            [[keyword]]
            word = "car"
            domain = "budha"
        "#;
        let lex = Lexicon::load_toml(toml).unwrap();
        assert!(lex.lookup("car").is_some());
        // "cardiac" contains "car" as a substring but must NOT match —
        // guards the same class of bug DOMAIN_KEYWORDS explicitly avoids.
        assert!(lex.lookup("cardiac").is_none());
        assert!(lex.lookup("scar").is_none());
    }

    #[test]
    fn rejects_empty_word() {
        let toml = "[[keyword]]\nword = \"\"\ndomain = \"budha\"\n";
        assert!(Lexicon::load_toml(toml).is_err());
    }

    #[test]
    fn accepts_multi_word_phrase() {
        let toml = r#"
            [[keyword]]
            word = "able account"
            domain = "budha"
            weight = 0.7
        "#;
        let lex = Lexicon::load_toml(toml).unwrap();
        assert_eq!(lex.len(), 1);
        // Not a single-word entry:
        assert!(lex.lookup("able").is_none());
        // But the phrase resolves when the exact token span is offered:
        let hit = lex.lookup_phrase(&["able", "account"]).unwrap();
        assert_eq!(hit.0, 2);
        assert_eq!(hit.1.domain, Domain::Budha);
        assert_eq!(hit.1.weight, 0.7);
    }

    #[test]
    fn phrase_longest_span_wins() {
        let toml = r#"
            [[keyword]]
            word = "personal finance plan"
            domain = "budha"

            [[keyword]]
            word = "personal finance"
            domain = "mangala"
        "#;
        let lex = Lexicon::load_toml(toml).unwrap();
        // Offering all three tokens must bind the 3-word phrase (budha), not
        // the 2-word prefix (mangala).
        let hit = lex.lookup_phrase(&["personal", "finance", "plan"]).unwrap();
        assert_eq!(hit.0, 3);
        assert_eq!(hit.1.domain, Domain::Budha);
    }

    #[test]
    fn phrase_requires_exact_contiguous_span() {
        let toml = "[[keyword]]\nword = \"able account\"\ndomain = \"budha\"\n";
        let lex = Lexicon::load_toml(toml).unwrap();
        // No phrase starts at "account", and "able" alone is not a phrase.
        assert!(lex.lookup_phrase(&["account"]).is_none());
        assert!(lex.lookup_phrase(&["able"]).is_none());
        // Non-adjacent / reordered tokens don't match.
        assert!(lex.lookup_phrase(&["able", "x", "account"]).is_none());
        assert!(lex.lookup_phrase(&["account", "able"]).is_none());
    }

    #[test]
    fn whitespace_normalized_phrase_key() {
        let toml = "[[keyword]]\nword = \"able  account\"\ndomain = \"budha\"\n";
        let lex = Lexicon::load_toml(toml).unwrap();
        assert!(lex.lookup_phrase(&["able", "account"]).is_some());
    }

    #[test]
    fn rejects_unknown_domain() {
        let toml = "[[keyword]]\nword = \"ssi\"\ndomain = \"pluto\"\n";
        assert!(Lexicon::load_toml(toml).is_err());
    }

    #[test]
    fn rejects_out_of_range_weight() {
        let toml = "[[keyword]]\nword = \"ssi\"\ndomain = \"budha\"\nweight = 1.5\n";
        assert!(Lexicon::load_toml(toml).is_err());
        let toml_zero = "[[keyword]]\nword = \"ssi\"\ndomain = \"budha\"\nweight = 0.0\n";
        assert!(Lexicon::load_toml(toml_zero).is_err());
    }

    #[test]
    fn rejects_duplicate_word() {
        let toml = r#"
            [[keyword]]
            word = "ssi"
            domain = "budha"

            [[keyword]]
            word = "SSI"
            domain = "mangala"
        "#;
        let err = Lexicon::load_toml(toml).unwrap_err();
        assert!(err.contains("duplicate"), "got: {err}");
    }

    #[test]
    fn empty_toml_yields_empty_lexicon() {
        let lex = Lexicon::load_toml("").unwrap();
        assert!(lex.is_empty());
    }
}
