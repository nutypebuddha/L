//! Golden-set regression tests for the route purity fixes (T54 / T55).
//!
//! T54: token→graha must be a PURE function of the token text — no neighbor
//!      domain inheritance (query-global contamination), and stopwords/unknown
//!      words must not invent a graha. The previously-contaminated graha was
//!      Mangala (a token's force could be smeared from a neighbor).
//! T55: stopwords carry no vote; surviving tokens are weighted by corpus
//!      specificity instead of a flat 1.0.
//!
//! These exercise the real CLI so the wired-up pipeline (descent → pure
//! resolution → synthesis) is covered end-to-end.

use std::process::Command;

use serde_json::Value;

fn route_json(query: &str) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_laverna"))
        .args(["route", "--query", query, "--format", "json"])
        .output()
        .expect("spawn laverna");
    let stdout = String::from_utf8(out.stdout).expect("stdout is utf-8");
    serde_json::from_str(&stdout).expect("route output is JSON")
}

fn force_names(report: &Value) -> Vec<String> {
    report["forces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap().to_string())
        .collect()
}

fn stopword_list(report: &Value) -> Vec<String> {
    report["stopwords"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn golden_query_never_routes_to_mangala_and_excludes_stopwords() {
    let report = route_json("how do i know what i don't know");

    // T54 core: the previously-contaminated graha (Mangala) must never win, and
    // must not appear anywhere in the ranked forces.
    assert_ne!(
        report["primary"]["name"].as_str(),
        Some("Mangala"),
        "contaminated graha Mangala must never be the primary route"
    );
    let forces = force_names(&report);
    assert!(
        !forces.contains(&"Mangala".to_string()),
        "Mangala must not appear in the ranked forces: {forces:?}"
    );

    // T55: every function word is filtered out before scoring.
    let stopwords = stopword_list(&report);
    for sw in ["how", "do", "i", "what", "don't"] {
        assert!(
            stopwords.contains(&sw.to_string()),
            "expected '{sw}' in stopwords, got {stopwords:?}"
        );
    }

    // The content token "know" is resolved, not dropped as a stopword or left
    // unresolved.
    assert!(
        !stopwords.contains(&"know".to_string()),
        "'know' is a content token, not a stopword"
    );
    let unresolved: Vec<String> = report["unresolved"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u.as_str().unwrap().to_string())
        .collect();
    assert!(
        !unresolved.contains(&"know".to_string()),
        "'know' resolved to a corpus graha, not unresolved"
    );

    // Determinism: identical query → identical primary force.
    let again = route_json("how do i know what i don't know");
    assert_eq!(report["primary"], again["primary"]);
}

#[test]
fn stopwords_do_not_pollute_routing() {
    // The same content tokens with and without surrounding function words must
    // route identically (T54: no neighbor domain inheritance; T55: stopwords
    // carry no vote). This is corpus-agnostic — it only asserts stopwords are
    // inert, not which graha wins.
    let with_stopwords = route_json("how do i compute the math");
    let bare = route_json("compute math");
    assert_eq!(
        with_stopwords["primary"], bare["primary"],
        "adding stopwords must not change the routed force"
    );
    assert!(
        with_stopwords["primary"]["name"].is_string(),
        "a clear content keyword must still route to a force"
    );
    // The stopwords were present and excluded from scoring.
    let stopwords = stopword_list(&with_stopwords);
    for sw in ["how", "do", "i", "the"] {
        assert!(
            stopwords.contains(&sw.to_string()),
            "'{sw}' should be a stopword"
        );
    }
    // The previously-contaminated graha must never be introduced by stopwords.
    assert_ne!(with_stopwords["primary"]["name"].as_str(), Some("Mangala"));
}

#[test]
fn out_of_corpus_query_fails_loud() {
    // No token maps to a corpus graha → fail loud: refused with a typed reason,
    // no invented primary strategy.
    let report = route_json("xyzzy qwerty plugh");
    assert!(
        report["primary"].is_null(),
        "out-of-corpus query must not invent a primary strategy"
    );
    assert_eq!(
        report["refused"].as_bool(),
        Some(true),
        "out-of-corpus query must be refused"
    );
    assert_eq!(
        report["kind"].as_str(),
        Some("OutOfScope"),
        "out-of-corpus query must carry a typed OutOfScope refusal"
    );
}
