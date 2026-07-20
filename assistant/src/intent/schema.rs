use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    /// "set a 5 minute timer for tea"
    SetTimer {
        duration_secs: u64,
        label: Option<String>,
    },
    /// "cancel the tea timer"
    CancelTimer { label: Option<String> },
    /// "set an alarm for 7am"
    SetAlarm { time: String },
    /// "remind me to call mom at 3pm"
    SetReminder { text: String, when: String },
    /// "remember that my favorite language is Rust" — durable user fact
    Remember { key: String, value: String },
    /// "recall my favorite language" / "what do you remember" — fetch fact(s)
    Recall { key: String },
    /// "forget my favorite language" — delete a durable fact
    Forget { key: String },
    /// "text John hello I'm on my way"
    #[cfg(feature = "termux")]
    SendMessage { contact: String, message: String },
    /// "call Sarah"
    #[cfg(feature = "termux")]
    Call { contact: String },
    /// "take a photo" / "snap a picture"
    #[cfg(feature = "termux")]
    TakePhoto,
    /// "what's my battery level" / "how much battery do I have"
    #[cfg(feature = "termux")]
    BatteryStatus,
    /// "where am I" / "what's my location"
    #[cfg(feature = "termux")]
    GetLocation,
    /// "copy this to clipboard"
    #[cfg(feature = "termux")]
    SetClipboard { text: String },
    /// "read clipboard" / "what's on my clipboard"
    #[cfg(feature = "termux")]
    GetClipboard,

    // ── Ecosystem intents ─────────────────────────────────────
    /// "prove that 2+2=4" / "reason about climate change" — Proof engine
    Solve { query: String },
    /// "validate this claim" / "check if water is H2O" — Gate validation
    Validate { text: String, context: String },
    /// "score this response" / "how confident are you" — Gate scoring
    Score { text: String },
    /// "fix this response" — Gate auto-fix
    Fix { text: String, context: String },
    /// "calculate 2+2" / "what's sqrt(144)" / "compute the area" — Tanto eval
    Eval { expression: String },
    /// "convert 100 miles to kilometers" — Tanto unit conversion
    Convert {
        value: f64,
        from: String,
        to: String,
    },
    /// "evaluate the circle_area formula with r=5" — Tanto named formula
    Formula { name: String, args: Vec<String> },
    /// "search formulas for gravity" / "find formulas about energy" — Athena
    SearchFormulas { keyword: String },
    /// "traverse the mangala domain" / "explore the aries wheel" — Athena
    Traverse { domain: String, depth: usize },
    /// "classify mercury" / "what axis is jupiter" — Athena
    Classify { token: String },
    /// "show the wheel" / "what's on the zodiac wheel" — Athena
    Wheel { domain: Option<String> },
    /// "reason from acceleration to distance" / "derive velocity from force" — Athena
    Reason {
        have: String,
        want: String,
        max_depth: usize,
    },
    /// "process this through shikai" — Athena Shikai NLP pipeline
    Shikai { query: String },
    /// "full bankai solve for X" — Athena Bankai full pipeline
    BankaiSolve { query: String },
    /// "evaluate formula circle_area with r=5" — Athena formula evaluation
    EvalFormula { formula_id: String, args: String },
    /// "chain formulas circle_area, sphere_volume" — Athena formula chaining
    ChainFormulas { formulas: String, args: String },

    // ── Generic fallbacks ─────────────────────────────────────
    /// "what is the capital of France" — routes to Proof reasoning engine
    Query { text: String },
    /// Open-ended conversational input — routes to LLM
    Conversational { text: String },
    /// "goodbye" / "bye" / "see you"
    Goodbye,
    /// "help" / "what can you do"
    Help,
    /// Fallback when nothing matches
    Unknown,
}
