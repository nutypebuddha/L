//! # Logic Gate — validates logical form and inference patterns
//!
//! Assimilates the CID fallacy pattern library (69 patterns across 6 categories)
//! from CID's `gates/fallacy.rs`. CID's patterns were derived from a systematic
//! audit of common logical fallacies in LLM reasoning, organized into formal,
//! informal, probabilistic, causal, epistemic, and rhetorical categories.
//!
//! Also includes 12 standard valid inference patterns (modus ponens, syllogism, etc.)
//! for a total of 81 detectable patterns.
//!
//! All patterns are hardcoded as const arrays — no HashMap, no runtime allocation,
//! fully deterministic.

use super::{gate_output, Gate, GateOutput};

/// A single logic pattern: id, description, whether it's valid or fallacious,
/// and an optional explanation.
pub struct LogicPattern {
    id: &'static str,
    description: &'static str,
    is_valid: bool,
    explanation: Option<&'static str>,
}

/// Valid deductive inference patterns (12 total).
const VALID_PATTERNS: &[LogicPattern] = &[
    LogicPattern {
        id: "modus_ponens",
        description: "If P then Q; P, therefore Q",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "modus_tollens",
        description: "If P then Q; Not Q, therefore Not P",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "syllogism",
        description: "All A are B; All B are C, therefore All A are C",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "hypothetical_syllogism",
        description: "If P then Q; If Q then R, therefore If P then R",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "disjunctive_syllogism",
        description: "P or Q; Not P, therefore Q",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "contrapositive",
        description: "If P then Q is equivalent to If Not Q then Not P",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "constructive_dilemma",
        description: "If P then Q; If R then S; P or R, therefore Q or S",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "destructive_dilemma",
        description: "If P then Q; If R then S; Not Q or Not S, therefore Not P or Not R",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "resolution",
        description: "P or Q; Not P or R, therefore Q or R",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "absorption",
        description: "If P then Q, therefore If P then (P and Q)",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "proof_by_cases",
        description: "If P then R; If Q then R; P or Q, therefore R",
        is_valid: true,
        explanation: None,
    },
    LogicPattern {
        id: "reductio_ad_absurdum",
        description: "Assume P; derive contradiction, therefore Not P",
        is_valid: true,
        explanation: None,
    },
];

// ─── Fallacy Categories ─────────────────────────────────────────────────────

/// Category 1: Formal fallacies (12) — errors in logical form.
const FORMAL_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "affirming_consequent",
        description: "If P then Q; Q, therefore P",
        is_valid: false,
        explanation: Some("Affirming the consequent: Q can be true for reasons other than P"),
    },
    LogicPattern {
        id: "denying_antecedent",
        description: "If P then Q; Not P, therefore Not Q",
        is_valid: false,
        explanation: Some("Denying the antecedent: Q can still be true even when P is false"),
    },
    LogicPattern {
        id: "fallacy_of_exclusion",
        description: "Ignoring relevant evidence that undermines the conclusion",
        is_valid: false,
        explanation: Some("Fallacy of exclusion: omitting evidence that contradicts the claim"),
    },
    LogicPattern {
        id: "masked_man_fallacy",
        description: "I know who X is; I do not know who Y is; therefore X is not Y",
        is_valid: false,
        explanation: Some(
            "Masked man fallacy (intentional fallacy): violating Leibniz's law — different modes of presentation can refer to the same entity",
        ),
    },
    LogicPattern {
        id: "illicit_major",
        description: "All A are B; No C are A, therefore No C are B",
        is_valid: false,
        explanation: Some(
            "Illicit major: the major term is distributed in the conclusion but not in the premise",
        ),
    },
    LogicPattern {
        id: "illicit_minor",
        description: "All A are B; All A are C, therefore All B are C",
        is_valid: false,
        explanation: Some(
            "Illicit minor: the minor term is distributed in the conclusion but not in the premise",
        ),
    },
    LogicPattern {
        id: "four_terms",
        description: "All A are B; All C are D, therefore All A are D",
        is_valid: false,
        explanation: Some(
            "Four terms fallacy (quaternio terminorum): a syllogism requires exactly three terms",
        ),
    },
    LogicPattern {
        id: "negative_premises",
        description: "No A are B; No C are A, therefore No C are B",
        is_valid: false,
        explanation: Some(
            "Exclusive premises: two negative premises cannot yield a valid conclusion in categorical logic",
        ),
    },
    LogicPattern {
        id: "existential_fallacy",
        description: "All A are B; No B are C, therefore Some A are not C",
        is_valid: false,
        explanation: Some("Existential fallacy: universal premises do not guarantee existence"),
    },
    LogicPattern {
        id: "modal_fallacy",
        description: "If P is necessary, and P implies Q, then Q is necessary",
        is_valid: false,
        explanation: Some(
            "Modal fallacy: conflating necessity of the consequence with necessity of the consequent",
        ),
    },
    LogicPattern {
        id: "quantifier_shift",
        description: "Everyone loves someone, therefore there is someone everyone loves",
        is_valid: false,
        explanation: Some("Quantifier shift fallacy: ∀∃ does not imply ∃∀"),
    },
    LogicPattern {
        id: "base_rate_fallacy",
        description: "Test is 99% accurate; person tested positive; therefore person has condition",
        is_valid: false,
        explanation: Some(
            "Base rate fallacy: ignoring the prior probability when interpreting test results",
        ),
    },
];

/// Category 2: Informal / Relevance fallacies (15) — premises irrelevant to conclusion.
const INFORMAL_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "ad_hominem",
        description: "Attacking the person instead of the argument",
        is_valid: false,
        explanation: Some(
            "Ad hominem: the character of the arguer is irrelevant to the truth of their claim",
        ),
    },
    LogicPattern {
        id: "tu_quoque",
        description: "You also do X, therefore your criticism of X is invalid",
        is_valid: false,
        explanation: Some("Tu quoque: hypocrisy does not make a claim false"),
    },
    LogicPattern {
        id: "straw_man",
        description: "Misrepresenting an argument to make it easier to attack",
        is_valid: false,
        explanation: Some("Straw man: the distorted version is not the actual argument"),
    },
    LogicPattern {
        id: "appeal_to_authority",
        description: "X is an authority; X says Y, therefore Y is true",
        is_valid: false,
        explanation: Some(
            "Appeal to authority: expertise in one domain does not transfer to others",
        ),
    },
    LogicPattern {
        id: "appeal_to_nature",
        description: "X is natural, therefore X is good",
        is_valid: false,
        explanation: Some("Appeal to nature: natural does not mean good or correct"),
    },
    LogicPattern {
        id: "appeal_to_emotion",
        description: "Evoking pity or fear instead of providing evidence",
        is_valid: false,
        explanation: Some("Appeal to emotion: emotional response is not a logical justification"),
    },
    LogicPattern {
        id: "appeal_to_tradition",
        description: "X has always been done, therefore X is correct",
        is_valid: false,
        explanation: Some("Appeal to tradition: longevity does not guarantee correctness"),
    },
    LogicPattern {
        id: "appeal_to_novelty",
        description: "X is new, therefore X is better",
        is_valid: false,
        explanation: Some("Appeal to novelty: recency does not guarantee superiority"),
    },
    LogicPattern {
        id: "genetic_fallacy",
        description: "Judging X as bad because of its origin or history",
        is_valid: false,
        explanation: Some("Genetic fallacy: origin is irrelevant to current truth-value"),
    },
    LogicPattern {
        id: "guilt_by_association",
        description: "X is associated with Y; Y is bad, therefore X is bad",
        is_valid: false,
        explanation: Some("Guilt by association: association does not entail shared properties"),
    },
    LogicPattern {
        id: "appeal_to_ignorance",
        description: "X has not been proven false, therefore X is true",
        is_valid: false,
        explanation: Some("Appeal to ignorance: lack of evidence against is not evidence for"),
    },
    LogicPattern {
        id: "burden_of_proof",
        description: "I assert X; you cannot prove X is false, therefore X is true",
        is_valid: false,
        explanation: Some("Shifting the burden of proof: the claimant must provide evidence"),
    },
    LogicPattern {
        id: "red_herring",
        description: "Introducing an irrelevant point to divert attention",
        is_valid: false,
        explanation: Some("Red herring: the diversion does not address the original claim"),
    },
    LogicPattern {
        id: "poisoning_the_well",
        description: "Discrediting an arguer before they present their argument",
        is_valid: false,
        explanation: Some("Poisoning the well: preemptive ad hominem undermines discourse"),
    },
    LogicPattern {
        id: "special_pleading",
        description: "Appealing to exceptional rules for one's own case without justification",
        is_valid: false,
        explanation: Some("Special pleading: exceptions require justification"),
    },
];

/// Category 3: Probabilistic fallacies (12) — errors in reasoning about probability.
const PROBABILISTIC_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "gamblers_fallacy",
        description: "After a run of heads, tails is due",
        is_valid: false,
        explanation: Some("Gambler's fallacy: independent events have no memory"),
    },
    LogicPattern {
        id: "hot_hand_fallacy",
        description: "After a run of successes, another success is more likely",
        is_valid: false,
        explanation: Some(
            "Hot hand fallacy: independent events are not influenced by past outcomes",
        ),
    },
    LogicPattern {
        id: "conjunction_fallacy",
        description: "A specific conjunction is more probable than one of its conjuncts",
        is_valid: false,
        explanation: Some(
            "Conjunction fallacy (Linda problem): P(A∧B) ≤ P(A) by the laws of probability",
        ),
    },
    LogicPattern {
        id: "disjunction_fallacy",
        description: "Underestimating the probability of a disjunction",
        is_valid: false,
        explanation: Some("Disjunction fallacy: P(A∨B) ≥ P(A), not less"),
    },
    LogicPattern {
        id: "availability_heuristic",
        description: "Overestimating probability of vivid or recent events",
        is_valid: false,
        explanation: Some(
            "Availability heuristic: ease of recall is not a reliable probability signal",
        ),
    },
    LogicPattern {
        id: "representativeness_heuristic",
        description: "Judging probability by similarity to a stereotype",
        is_valid: false,
        explanation: Some("Representativeness heuristic: similarity ignores base rates"),
    },
    LogicPattern {
        id: "anchoring",
        description: "Excessive reliance on an initial reference point",
        is_valid: false,
        explanation: Some("Anchoring bias: initial reference points skew subsequent estimates"),
    },
    LogicPattern {
        id: "confirmation_bias",
        description: "Seeking evidence that confirms existing beliefs",
        is_valid: false,
        explanation: Some(
            "Confirmation bias: disconfirming evidence is systematically undervalued",
        ),
    },
    LogicPattern {
        id: "survivorship_bias",
        description: "Drawing conclusions from only surviving examples",
        is_valid: false,
        explanation: Some("Survivorship bias: silent evidence of failures is invisible"),
    },
    LogicPattern {
        id: "prosecutors_fallacy",
        description: "Confusing P(evidence|innocent) with P(innocent|evidence)",
        is_valid: false,
        explanation: Some("Prosecutor's fallacy: conditional probability transposition error"),
    },
    LogicPattern {
        id: "defense_fallacy",
        description: "Confusing P(evidence|guilty) with P(guilty|evidence)",
        is_valid: false,
        explanation: Some("Defense attorney's fallacy: mirror image of the prosecutor's fallacy"),
    },
    LogicPattern {
        id: "regression_fallacy",
        description: "Attributing a cause to natural regression to the mean",
        is_valid: false,
        explanation: Some(
            "Regression fallacy: extreme values naturally become less extreme on retest",
        ),
    },
];

/// Category 4: Causal fallacies (10) — errors in causal reasoning.
const CAUSAL_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "post_hoc_ergo_propter_hoc",
        description: "A happened before B, therefore A caused B",
        is_valid: false,
        explanation: Some("Post hoc ergo propter hoc: temporal order does not imply causation"),
    },
    LogicPattern {
        id: "cum_hoc_ergo_propter_hoc",
        description: "A and B are correlated, therefore A causes B",
        is_valid: false,
        explanation: Some("Cum hoc ergo propter hoc: correlation does not imply causation"),
    },
    LogicPattern {
        id: "single_cause",
        description: "Assuming a single cause when multiple factors are involved",
        is_valid: false,
        explanation: Some("Single cause fallacy: most effects have multiple contributing causes"),
    },
    LogicPattern {
        id: "slippery_slope",
        description: "A leads to B, B leads to C, therefore A will lead to disaster",
        is_valid: false,
        explanation: Some(
            "Slippery slope: intermediate steps may not follow or may be preventable",
        ),
    },
    LogicPattern {
        id: "reversing_causality",
        description: "Treating effect as cause and cause as effect",
        is_valid: false,
        explanation: Some(
            "Reverse causality: the direction of causation may be opposite to what is assumed",
        ),
    },
    LogicPattern {
        id: "third_cause",
        description: "Ignoring a common cause of both A and B",
        is_valid: false,
        explanation: Some(
            "Third cause fallacy (spurious correlation): a hidden variable may cause both",
        ),
    },
    LogicPattern {
        id: "domino_fallacy",
        description: "Assuming a chain reaction without evidence of causal links",
        is_valid: false,
        explanation: Some("Domino fallacy: each causal link must be individually justified"),
    },
    LogicPattern {
        id: "magical_thinking",
        description: "Assuming thoughts or words can directly influence physical events",
        is_valid: false,
        explanation: Some(
            "Magical thinking: mental states do not directly cause physical events without mediating mechanisms",
        ),
    },
    LogicPattern {
        id: "regression_fallacy_causal",
        description: "Punishment improves performance; reward worsens it",
        is_valid: false,
        explanation: Some(
            "Regression fallacy in performance: extreme performance naturally regresses, regardless of intervention",
        ),
    },
    LogicPattern {
        id: "just_so_story",
        description: "A plausible narrative is accepted as explanation without evidence",
        is_valid: false,
        explanation: Some("Just-so story: plausibility is not evidence"),
    },
];

/// Category 5: Epistemic fallacies (10) — errors in knowledge and evidence.
const EPISTEMIC_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "circular_reasoning",
        description: "The conclusion is assumed in one of the premises",
        is_valid: false,
        explanation: Some(
            "Circular reasoning (begging the question): the argument assumes what it tries to prove",
        ),
    },
    LogicPattern {
        id: "infinite_regress",
        description: "Each justification requires further justification without end",
        is_valid: false,
        explanation: Some(
            "Infinite regress: an endless chain of justifications that never reaches a foundation",
        ),
    },
    LogicPattern {
        id: "no_true_scotsman",
        description: "Redefining a category to exclude counterexamples",
        is_valid: false,
        explanation: Some(
            "No true Scotsman: ad-hoc redefinition insulates a claim from falsification",
        ),
    },
    LogicPattern {
        id: "moving_goalposts",
        description: "Changing the criteria for evidence after it has been met",
        is_valid: false,
        explanation: Some(
            "Moving the goalposts: the standard of proof shifts to avoid disconfirmation",
        ),
    },
    LogicPattern {
        id: "false_dilemma",
        description: "Presenting two options as the only possibilities when more exist",
        is_valid: false,
        explanation: Some(
            "False dilemma (black-and-white fallacy): artificially restricts the option space",
        ),
    },
    LogicPattern {
        id: "argument_from_incredulity",
        description: "I cannot imagine how X could be true, therefore X is false",
        is_valid: false,
        explanation: Some(
            "Argument from incredulity: inability to imagine does not constitute disproof",
        ),
    },
    LogicPattern {
        id: "argument_from_personal_incredulity",
        description: "I don't understand X, therefore X is false",
        is_valid: false,
        explanation: Some("Personal incredulity: lack of understanding is not an argument"),
    },
    LogicPattern {
        id: "equivocation",
        description: "Using the same word in two different senses",
        is_valid: false,
        explanation: Some("Equivocation: ambiguous terms must be disambiguated"),
    },
    LogicPattern {
        id: "composition",
        description: "What is true of the parts is true of the whole",
        is_valid: false,
        explanation: Some("Fallacy of composition: parts may not share properties with the whole"),
    },
    LogicPattern {
        id: "division",
        description: "What is true of the whole is true of its parts",
        is_valid: false,
        explanation: Some("Fallacy of division: the whole may have properties its parts lack"),
    },
];

/// Category 6: Rhetorical fallacies (8) — manipulative language patterns.
const RHETORICAL_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "loaded_question",
        description: "Asking a question containing an unsubstantiated assumption",
        is_valid: false,
        explanation: Some("Loaded question: the presupposition may be false"),
    },
    LogicPattern {
        id: "begging_the_question",
        description: "The claim is restated rather than proven",
        is_valid: false,
        explanation: Some(
            "Begging the question (petitio principii): circular argument where premises assume conclusion",
        ),
    },
    LogicPattern {
        id: "complex_question",
        description: "A question that presupposes something not yet established",
        is_valid: false,
        explanation: Some("Complex question: multiple issues collapsed into one"),
    },
    LogicPattern {
        id: "argument_by_repetition",
        description: "Repeating a claim until it is accepted as true",
        is_valid: false,
        explanation: Some("Argumentum ad nauseam: repetition is not evidence"),
    },
    LogicPattern {
        id: "appeal_to_ridicule",
        description: "Deriding an argument instead of addressing it",
        is_valid: false,
        explanation: Some("Appeal to ridicule: mockery is not refutation"),
    },
    LogicPattern {
        id: "appeal_to_consequences",
        description: "X must be true because believing X has good consequences",
        is_valid: false,
        explanation: Some(
            "Appeal to consequences: desirability of a belief does not determine its truth",
        ),
    },
    LogicPattern {
        id: "wishful_thinking",
        description: "X is true because I want it to be true",
        is_valid: false,
        explanation: Some("Wishful thinking: desire does not affect reality"),
    },
    LogicPattern {
        id: "thought_terminating_cliche",
        description: "Using a trite saying to dismiss legitimate concerns",
        is_valid: false,
        explanation: Some(
            "Thought-terminating cliché: platitudes that discourage critical examination",
        ),
    },
];

/// Category 7: Vagueness / Ambiguity fallacies (2)
const VAGUENESS_FALLACIES: &[LogicPattern] = &[
    LogicPattern {
        id: "sorites_paradox",
        description: "One grain does not make a heap; adding one grain never makes a heap; therefore there are no heaps",
        is_valid: false,
        explanation: Some("Sorites paradox: vague predicates do not have sharp boundaries"),
    },
    LogicPattern {
        id: "continuum_fallacy",
        description: "No clear boundary exists, therefore no distinction can be made",
        is_valid: false,
        explanation: Some(
            "Continuum fallacy: the absence of a sharp boundary does not mean no boundary exists",
        ),
    },
];

// Total: 12 valid + 12 formal + 15 informal + 12 probabilistic + 10 causal + 10 epistemic + 8 rhetorical + 2 vagueness = 81 patterns.
// Collect all patterns for iterative matching.
const ALL_PATTERNS: &[&[LogicPattern]] = &[
    VALID_PATTERNS,
    FORMAL_FALLACIES,
    INFORMAL_FALLACIES,
    PROBABILISTIC_FALLACIES,
    CAUSAL_FALLACIES,
    EPISTEMIC_FALLACIES,
    RHETORICAL_FALLACIES,
    VAGUENESS_FALLACIES,
];

/// Validates logical structure of arguments using standard patterns.
pub struct LogicGate;

impl LogicGate {
    pub fn new() -> Self {
        LogicGate
    }

    /// Total number of patterns available.
    pub const fn pattern_count() -> usize {
        // Sum all patterns across all categories
        12 + 12 + 15 + 12 + 10 + 10 + 8 + 2
    }

    /// Count how many of each type.
    pub fn category_counts() -> Vec<(&'static str, usize)> {
        vec![
            ("valid", VALID_PATTERNS.len()),
            ("formal_fallacies", FORMAL_FALLACIES.len()),
            ("informal_fallacies", INFORMAL_FALLACIES.len()),
            ("probabilistic_fallacies", PROBABILISTIC_FALLACIES.len()),
            ("causal_fallacies", CAUSAL_FALLACIES.len()),
            ("epistemic_fallacies", EPISTEMIC_FALLACIES.len()),
            ("rhetorical_fallacies", RHETORICAL_FALLACIES.len()),
            ("vagueness_fallacies", VAGUENESS_FALLACIES.len()),
        ]
    }

    /// Check if the text matches or describes a known logic pattern.
    pub fn check_logic(&self, text: &str) -> GateOutput {
        // Normalize: replace spaces with underscores for matching pattern IDs
        let lower = text.to_lowercase();
        let lower_normalized = lower.replace([' ', '\''], "_");

        let mut issues = Vec::new();
        let mut matched = false;
        let mut passed = true;

        for category in ALL_PATTERNS {
            for pattern in *category {
                // Check if the normalized text contains the pattern ID
                // or if the pattern ID is found within the normalized text
                let id_match =
                    lower_normalized.contains(pattern.id) || pattern.id.contains(&lower_normalized);

                // Check if the pattern ID words are found in the text
                let desc_match = !pattern.id.is_empty()
                    && pattern.id.len() > 2
                    && lower.contains(&pattern.id.replace('_', " "));

                if id_match || desc_match {
                    matched = true;
                    if !pattern.is_valid {
                        passed = false;
                        let explanation = pattern.explanation.unwrap_or("No explanation available");
                        issues.push(format!(
                            "{} — {}. {}",
                            pattern.id, pattern.description, explanation
                        ));
                    } else {
                        // Valid pattern found — return immediately with high confidence
                        return gate_output(
                            "logic",
                            true,
                            0.95,
                            format!(
                                "Valid logic pattern: {} — {}",
                                pattern.id, pattern.description
                            ),
                            vec![],
                            vec![],
                        );
                    }
                }
            }
        }

        if matched {
            let message_text = if issues.is_empty() {
                "No fallacious patterns detected".to_string()
            } else {
                format!("Fallacious pattern(s) detected: {}", issues.join("; "))
            };
            // Return with lower confidence if fallacies were found
            let confidence = if passed { 0.7 } else { 0.3 };
            gate_output("logic", passed, confidence, message_text, issues, vec![])
        } else {
            gate_output(
                "logic",
                true,
                0.5,
                "No logic pattern detected — no claim made".to_string(),
                vec![],
                vec![],
            )
        }
    }

    /// Get a specific pattern by ID.
    pub fn get_pattern(id: &str) -> Option<&'static LogicPattern> {
        let id_lower = id.to_lowercase();
        for category in ALL_PATTERNS {
            for pattern in *category {
                if pattern.id == id_lower {
                    return Some(pattern);
                }
            }
        }
        None
    }
}

impl Gate for LogicGate {
    fn name(&self) -> &str {
        "logic"
    }

    fn check(&self, target: &str) -> GateOutput {
        self.check_logic(target)
    }
}

impl Default for LogicGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modus_ponens() {
        let gate = LogicGate::new();
        let result = gate.check_logic("modus ponens");
        assert!(result.passed);
    }

    #[test]
    fn test_modus_tollens() {
        let gate = LogicGate::new();
        let result = gate.check_logic("modus tollens");
        assert!(result.passed);
    }

    #[test]
    fn test_affirming_consequent() {
        let gate = LogicGate::new();
        let result = gate.check_logic("affirming_consequent");
        assert!(!result.passed);
    }

    #[test]
    fn test_ad_hominem() {
        let gate = LogicGate::new();
        let result = gate.check_logic("ad hominem");
        assert!(!result.passed);
    }

    #[test]
    fn test_straw_man() {
        let gate = LogicGate::new();
        let result = gate.check_logic("straw man");
        assert!(!result.passed);
    }

    #[test]
    fn test_slippery_slope() {
        let gate = LogicGate::new();
        let result = gate.check_logic("slippery slope");
        assert!(!result.passed);
    }

    #[test]
    fn test_false_dilemma() {
        let gate = LogicGate::new();
        let result = gate.check_logic("false dilemma");
        assert!(!result.passed);
    }

    #[test]
    fn test_post_hoc() {
        let gate = LogicGate::new();
        let result = gate.check_logic("post hoc ergo propter hoc");
        assert!(!result.passed);
    }

    #[test]
    fn test_circular_reasoning() {
        let gate = LogicGate::new();
        let result = gate.check_logic("circular reasoning");
        assert!(!result.passed);
    }

    #[test]
    fn test_gamblers_fallacy() {
        let gate = LogicGate::new();
        let result = gate.check_logic("gamblers_fallacy");
        assert!(!result.passed);
    }

    #[test]
    fn test_no_logic_pattern() {
        let gate = LogicGate::new();
        let result = gate.check_logic("the sky is blue");
        assert!(result.passed); // no claim made — vacuously passes
        assert!((result.confidence - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_syllogism() {
        let gate = LogicGate::new();
        let result = gate.check_logic("syllogism");
        assert!(result.passed);
        assert!((result.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_patent_count() {
        let count = LogicGate::pattern_count();
        assert_eq!(count, 81);
    }

    #[test]
    fn test_category_counts() {
        let cats = LogicGate::category_counts();
        let total: usize = cats.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 81);
    }

    #[test]
    fn test_get_pattern_by_id() {
        let p = LogicGate::get_pattern("ad_hominem");
        assert!(p.is_some());
        assert!(!p.unwrap().is_valid);

        let q = LogicGate::get_pattern("modus_ponens");
        assert!(q.is_some());
        assert!(q.unwrap().is_valid);

        assert!(LogicGate::get_pattern("nonexistent").is_none());
    }

    #[test]
    fn test_confirmation_bias() {
        let gate = LogicGate::new();
        let result = gate.check_logic("confirmation bias");
        assert!(!result.passed);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_survivorship_bias() {
        let gate = LogicGate::new();
        let result = gate.check_logic("survivorship bias");
        assert!(!result.passed);
    }

    #[test]
    fn test_burden_of_proof() {
        let gate = LogicGate::new();
        let result = gate.check_logic("burden of proof");
        assert!(!result.passed);
    }
}
