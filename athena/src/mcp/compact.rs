//! # Response compaction — token savings for the calling model
//!
//! Every byte Athena emits over MCP is read back as context tokens by the
//! LLM on the other end. This module shrinks responses before they hit the
//! wire and keeps a ledger of what the verbose rendering *would* have cost,
//! so the savings are measurable (`savings` tool).
//!
//! Compaction is lossy but principled:
//! - lists are capped at `limit` entries (an `omitted` count is added)
//! - descriptions lose their cosmological flavor suffix (`" — ..."`)
//! - per-entry fields are trimmed to what's needed to pick a next action
//! - `null` fields are pruned everywhere
//!
//! Pass `detail: "full"` on any tool call to bypass compaction.

use serde_json::{json, Map, Value};

/// Default cap on list-shaped responses (`formula_search`, `entity_list`, ...).
pub const DEFAULT_LIMIT: usize = 25;

/// How much of a response the caller wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// Trimmed for token economy (default).
    Compact,
    /// The complete response, as `handle_request` produced it.
    Full,
}

impl Detail {
    /// Parse from tool-call arguments; anything other than `"full"` is compact.
    pub fn from_params(params: Option<&Map<String, Value>>) -> Self {
        match params
            .and_then(|p| p.get("detail"))
            .and_then(|v| v.as_str())
        {
            Some("full") => Detail::Full,
            _ => Detail::Compact,
        }
    }
}

/// Cumulative record of tokens emitted vs. what verbose output would have cost.
#[derive(Debug, Default, Clone)]
pub struct SavingsLedger {
    /// Tool calls that flowed through the compaction layer.
    pub calls: u64,
    /// Estimated tokens the old pretty-printed full responses would have cost.
    pub baseline_tokens: usize,
    /// Estimated tokens actually emitted.
    pub emitted_tokens: usize,
}

impl SavingsLedger {
    /// Record one response: baseline (verbose) vs. emitted (actual) estimate.
    pub fn record(&mut self, baseline: usize, emitted: usize) {
        self.calls += 1;
        self.baseline_tokens += baseline;
        self.emitted_tokens += emitted;
    }

    /// Tokens saved so far (never negative).
    pub fn saved(&self) -> usize {
        self.baseline_tokens.saturating_sub(self.emitted_tokens)
    }

    /// Fraction of baseline tokens saved, 0.0 when nothing recorded.
    pub fn saved_pct(&self) -> f64 {
        if self.baseline_tokens == 0 {
            0.0
        } else {
            self.saved() as f64 / self.baseline_tokens as f64
        }
    }
}

/// Estimate tokens for a text (same ~4 chars/token + 10% margin heuristic
/// as `budget::TokenBudget::estimate_tokens` for natural language).
pub fn estimate_tokens(text: &str) -> usize {
    ((text.len() as f64 / 4.0) * 1.1).ceil() as usize
}

/// Strip the cosmological flavor suffix from a description:
/// `"Energy conservation: E_i = E_f — Fixed Earth: Venus harmony"` →
/// `"Energy conservation: E_i = E_f"`.
fn strip_flavor(s: &str) -> &str {
    s.split(" — ").next().unwrap_or(s).trim()
}

/// Cap `data[key]` at `limit` entries, recording the omission count in
/// `data["omitted"]` so the caller knows to raise `limit` or use `detail: "full"`.
fn truncate_list(data: &mut Value, key: &str, limit: usize) {
    let omitted = match data.get_mut(key).and_then(|v| v.as_array_mut()) {
        Some(arr) if arr.len() > limit => {
            let omitted = arr.len() - limit;
            arr.truncate(limit);
            omitted
        }
        _ => return,
    };
    data["omitted"] = json!(omitted);
}

/// Keep only `keys` in each object of the array at `data[key]`, applying
/// `strip_flavor` to any retained `description`.
fn slim_entries(data: &mut Value, key: &str, keys: &[&str]) {
    if let Some(arr) = data.get_mut(key).and_then(|v| v.as_array_mut()) {
        for entry in arr.iter_mut() {
            if let Value::Object(obj) = entry {
                obj.retain(|k, _| keys.contains(&k.as_str()));
                if let Some(Value::String(desc)) = obj.get_mut("description") {
                    *desc = strip_flavor(desc).to_string();
                }
            }
        }
    }
}

/// Recursively drop `null` members from objects.
fn prune_nulls(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            obj.retain(|_, v| !v.is_null());
            obj.values_mut().for_each(prune_nulls);
        }
        Value::Array(arr) => arr.iter_mut().for_each(prune_nulls),
        _ => {}
    }
}

/// Compact a successful `handle_request` response for the given method.
///
/// `method` is the inner method name (no `athena_` prefix). Unknown methods
/// still get the generic null-prune, which is always safe.
pub fn compact_data(method: &str, mut data: Value, limit: usize) -> Value {
    match method {
        "formula_search" => {
            slim_entries(&mut data, "formulas", &["id", "domain", "description"]);
            truncate_list(&mut data, "formulas", limit);
        }
        "formula_by_output" => {
            slim_entries(
                &mut data,
                "formulas",
                &[
                    "id",
                    "domain",
                    "description",
                    "inputs",
                    "output",
                    "expression",
                ],
            );
            truncate_list(&mut data, "formulas", limit);
        }
        "entity_list" => {
            slim_entries(&mut data, "entities", &["id", "name", "kind", "tags"]);
            truncate_list(&mut data, "entities", limit);
        }
        "entity_search" => {
            slim_entries(
                &mut data,
                "entities",
                &["id", "name", "text", "kind", "description", "graha"],
            );
            truncate_list(&mut data, "entities", limit);
        }
        "classify" => {
            // The full per-axis score maps are the bulk of the payload;
            // the dominant picks are what the caller acts on.
            let confidence = data
                .get("vedic")
                .and_then(|v| v.get("confidence"))
                .cloned()
                .unwrap_or(Value::Null);
            let mut slim = json!({
                "text": data.get("text").cloned().unwrap_or(Value::Null),
                "dominant": data.get("dominant").cloned().unwrap_or(Value::Null),
            });
            if !confidence.is_null() {
                slim["confidence"] = confidence;
            }
            data = slim;
        }
        "wheel" => {
            // Full-wheel dump: symbol/name/opposite is enough to navigate.
            slim_entries(&mut data, "domains", &["symbol", "name", "opposite"]);
        }
        _ => {}
    }
    prune_nulls(&mut data);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_flavor() {
        assert_eq!(
            strip_flavor("Energy conservation: E_i = E_f — Fixed Earth: Venus harmony"),
            "Energy conservation: E_i = E_f"
        );
        assert_eq!(strip_flavor("no flavor here"), "no flavor here");
    }

    #[test]
    fn test_detail_from_params() {
        let full: Map<String, Value> = serde_json::from_value(json!({"detail": "full"})).unwrap();
        assert_eq!(Detail::from_params(Some(&full)), Detail::Full);
        let other: Map<String, Value> = serde_json::from_value(json!({"detail": "x"})).unwrap();
        assert_eq!(Detail::from_params(Some(&other)), Detail::Compact);
        assert_eq!(Detail::from_params(None), Detail::Compact);
    }

    #[test]
    fn test_truncate_list_adds_omitted() {
        let mut data = json!({"count": 4, "formulas": [1, 2, 3, 4]});
        truncate_list(&mut data, "formulas", 2);
        assert_eq!(data["formulas"].as_array().unwrap().len(), 2);
        assert_eq!(data["omitted"], 2);
    }

    #[test]
    fn test_truncate_list_under_limit_untouched() {
        let mut data = json!({"count": 2, "formulas": [1, 2]});
        truncate_list(&mut data, "formulas", 25);
        assert_eq!(data["formulas"].as_array().unwrap().len(), 2);
        assert!(data.get("omitted").is_none());
    }

    #[test]
    fn test_compact_formula_search_slims_and_caps() {
        let entry = json!({
            "id": "conservation_energy",
            "domain": "Shukra",
            "type": "Atomic",
            "description": "Energy conservation: E_i = E_f — Fixed Earth: harmony",
        });
        let mut formulas = Vec::new();
        for _ in 0..30 {
            formulas.push(entry.clone());
        }
        let data = json!({"count": 30, "formulas": formulas});
        let out = compact_data("formula_search", data, 25);
        let arr = out["formulas"].as_array().unwrap();
        assert_eq!(arr.len(), 25);
        assert_eq!(out["omitted"], 5);
        assert!(arr[0].get("type").is_none(), "type dropped in compact");
        assert_eq!(arr[0]["description"], "Energy conservation: E_i = E_f");
        assert_eq!(out["count"], 30, "total count preserved");
    }

    #[test]
    fn test_compact_classify_keeps_dominant_only() {
        let data = json!({
            "text": "fire",
            "classification": {"signs": {"Aries": 0.9}, "houses": {}},
            "dominant": {"sign": "Aries", "element": null},
            "vedic": {"grahas": {"Mangala": 0.8}, "confidence": 0.8},
        });
        let out = compact_data("classify", data, 25);
        assert_eq!(out["text"], "fire");
        assert_eq!(out["dominant"]["sign"], "Aries");
        assert_eq!(out["confidence"], 0.8);
        assert!(out.get("classification").is_none());
        assert!(out.get("vedic").is_none());
        assert!(
            out["dominant"].get("element").is_none(),
            "null dominants pruned"
        );
    }

    #[test]
    fn test_prune_nulls_recursive() {
        let mut data = json!({"a": null, "b": {"c": null, "d": 1}, "e": [ {"f": null} ]});
        prune_nulls(&mut data);
        assert!(data.get("a").is_none());
        assert!(data["b"].get("c").is_none());
        assert_eq!(data["b"]["d"], 1);
        assert!(data["e"][0].get("f").is_none());
    }

    #[test]
    fn test_unknown_method_passthrough() {
        let data = json!({"result": 42.0, "note": null});
        let out = compact_data("evaluate", data, 25);
        assert_eq!(out["result"], 42.0);
        assert!(out.get("note").is_none());
    }

    #[test]
    fn test_savings_ledger() {
        let mut ledger = SavingsLedger::default();
        ledger.record(1000, 300);
        ledger.record(500, 200);
        assert_eq!(ledger.calls, 2);
        assert_eq!(ledger.saved(), 1000);
        assert!((ledger.saved_pct() - 1000.0 / 1500.0).abs() < 1e-9);
    }

    #[test]
    fn test_estimate_tokens_heuristic() {
        // 40 chars / 4 per token * 1.1 = 11
        assert_eq!(estimate_tokens(&"x".repeat(40)), 11);
        assert_eq!(estimate_tokens(""), 0);
    }
}
