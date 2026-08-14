//! Runtime-loadable keyword → domain lexicon (dynamic strategy coverage).
//!
//! `DOMAIN_KEYWORDS` in `descent::mod` is a `const` compiled into the binary —
//! teaching the router a new topic (e.g. "personal finance") today means a
//! Rust PR and a rebuild. `Lexicon` is the same whole-word, deterministic,
//! weighted mapping, but loaded from an external TOML file at runtime, so an
//! "alien" domain can be handed to `route`/`strategize` in minutes via a small
//! TOML file instead of a recompile cycle.
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
//! word = "able_account"
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

/// A runtime-loaded, whole-word keyword → domain table.
///
/// Deliberately the simplest possible structure (exact lowercase word match,
/// no regex, no stemming) — extending it to reuse `stem_token` is a natural
/// T-ticket follow-up, but the MVP mirrors `domain_for_keyword`'s core
/// guarantee: a keyword never matches as a bare substring fragment, because
/// there is no substring matching here at all, only exact map lookup.
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    entries: HashMap<String, LexiconEntry>,
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
    /// silent partial loads) on: empty word, a word containing whitespace
    /// (phrases aren't supported by this exact-match MVP), an unparseable
    /// domain name, or a weight outside `(0.0, 1.0]`. A duplicate `word`
    /// across entries is also rejected rather than silently letting the
    /// last one win.
    pub fn load_toml(content: &str) -> Result<Self, String> {
        let parsed: LexiconFile =
            toml::from_str(content).map_err(|e| format!("lexicon TOML parse error: {e}"))?;

        let mut entries = HashMap::with_capacity(parsed.keyword.len());
        for kw in parsed.keyword {
            let word = kw.word.trim().to_lowercase();
            if word.is_empty() {
                return Err("lexicon entry has an empty `word`".to_string());
            }
            if word.split_whitespace().count() != 1 {
                return Err(format!(
                    "lexicon word {:?} must be a single whole word (no spaces) — \
                     this exact-match lexicon does not support phrases",
                    kw.word
                ));
            }
            let domain = Domain::parse(&kw.domain).ok_or_else(|| {
                format!(
                    "lexicon entry {word:?} has unrecognized domain {:?}",
                    kw.domain
                )
            })?;
            if !(kw.weight > 0.0 && kw.weight <= 1.0) {
                return Err(format!(
                    "lexicon entry {word:?} has weight {} — must be in (0.0, 1.0]",
                    kw.weight
                ));
            }
            if entries
                .insert(
                    word.clone(),
                    LexiconEntry {
                        domain,
                        weight: kw.weight,
                    },
                )
                .is_some()
            {
                return Err(format!("lexicon has duplicate entry for word {word:?}"));
            }
        }
        Ok(Lexicon { entries })
    }

    /// Whole-word, case-insensitive lookup. Returns `None` for anything not
    /// an exact match — never a substring hit.
    pub fn lookup(&self, token: &str) -> Option<LexiconEntry> {
        self.entries.get(&token.to_lowercase()).copied()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

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
    fn rejects_multi_word_phrase() {
        let toml = "[[keyword]]\nword = \"able account\"\ndomain = \"budha\"\n";
        let err = Lexicon::load_toml(toml).unwrap_err();
        assert!(err.contains("single whole word"), "got: {err}");
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
