/// Engine handles — closure-based interfaces to the L.ai ecosystem.
///
/// The assistant crate never depends on Proof, Gate, Athena, or Tanto directly.
/// Instead, `proof/src/main.rs` constructs an `Engines` with closures that call
/// into those subsystems and passes it to the assistant at launch.
use std::fmt;

type BoxFn<I, O> = Box<dyn Fn(I) -> O + Send + Sync>;

/// Handle to every L.ai subsystem the assistant can call.
pub struct Engines {
    /// Proof: full reasoning pipeline (descent → formula → Tanto evaluation).
    pub solve: BoxFn<String, String>,
    /// Gate: validate a response against context. (text, context) → report.
    pub validate: BoxFn<(String, String), String>,
    /// Gate: score a response 0-100. text → score string.
    pub score: BoxFn<String, String>,
    /// Gate: auto-fix a failing response. (text, context) → fixed text.
    pub fix: BoxFn<(String, String), String>,
    /// Tanto: evaluate a math/logic expression. expr → result string.
    pub eval: BoxFn<String, String>,
    /// Tanto: convert units. (value, from, to) → result string.
    pub convert: BoxFn<(f64, String, String), String>,
    /// Tanto: evaluate a named formula. (name, args) → result string.
    pub formula: BoxFn<(String, Vec<String>), String>,
    /// Athena: search formulas by keyword. keyword → result string.
    pub search_formulas: BoxFn<String, String>,
    /// Athena: traverse the zodiac wheel from a domain. (domain, depth) → result.
    pub traverse: BoxFn<(String, usize), String>,
    /// Athena: classify a token across 7 astrological axes. token → result.
    pub classify: BoxFn<String, String>,
    /// Athena: show the zodiac wheel. domain_opt → result string.
    pub wheel: BoxFn<Option<String>, String>,
    /// Athena: BFS path-finding. (have_csv, want, max_depth) → result.
    pub reason: BoxFn<(String, String, usize), String>,
    /// Athena: Shikai NLP pipeline. query → result string.
    pub shikai: BoxFn<String, String>,
    /// Athena: full Bankai solve pipeline. query → result string.
    pub bankai_solve: BoxFn<String, String>,
    /// Athena: evaluate a formula. (formula_id, args_csv) → result.
    pub eval_formula: BoxFn<(String, String), String>,
    /// Athena: chain formulas. (formulas_csv, args_csv) → result.
    pub chain_formulas: BoxFn<(String, String), String>,
}

impl Engines {
    /// No-op placeholder for testing — every call returns an empty string.
    pub fn noop() -> Self {
        Self {
            solve: Box::new(|q| format!("(no proof engine) {q}")),
            validate: Box::new(|(t, c)| format!("(no gate engine) validating '{t}' against '{c}'")),
            score: Box::new(|t| format!("(no gate engine) score for '{t}'")),
            fix: Box::new(|(t, _c)| t),
            eval: Box::new(|e| format!("(no tanto engine) {e}")),
            convert: Box::new(|(v, f, t)| format!("(no tanto engine) {v} {f} -> {t}")),
            formula: Box::new(|(n, _a)| format!("(no tanto engine) formula {n}")),
            search_formulas: Box::new(|k| format!("(no athena engine) search '{k}'")),
            traverse: Box::new(|(d, _dep)| format!("(no athena engine) traverse {d}")),
            classify: Box::new(|t| format!("(no athena engine) classify '{t}'")),
            wheel: Box::new(|d| format!("(no athena engine) wheel {d:?}")),
            reason: Box::new(|(h, w, _)| format!("(no athena engine) {h} -> {w}")),
            shikai: Box::new(|q| format!("(no athena engine) shikai '{q}'")),
            bankai_solve: Box::new(|q| format!("(no athena engine) bankai '{q}'")),
            eval_formula: Box::new(|(f, a)| format!("(no athena engine) eval {f} with {a}")),
            chain_formulas: Box::new(|(f, a)| format!("(no athena engine) chain {f} with {a}")),
        }
    }
}

impl fmt::Debug for Engines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Engines").finish_non_exhaustive()
    }
}
