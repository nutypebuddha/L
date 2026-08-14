pub mod bias;
pub mod cache;
pub mod compressor;
pub mod fixer;
pub mod json;
pub mod pipeline;
pub mod proxy;
pub mod request;
pub mod result;
pub mod sanity;
pub mod scorer;
pub mod stream;

pub use cache::{CacheHit, CacheStats, SemanticCache};
pub use compressor::{CompressionLevel, CompressionStats, PromptCompressor};
pub use fixer::TokenFixer;
pub use pipeline::Pipeline;
pub use proxy::ProxyServer;
pub use request::{InferenceConfig, ProxyConfig, ProxyRequest, ValidationRequest};
pub use result::{CidError, CidResult, GateScore, TokenFix, ValidationResult};
pub use scorer::{QualityReport, ResponseScorer, SuggestedAction};
pub use stream::{StreamEvent, StreamValidator};

use crate::core::ball::Ball;
use crate::core::pin::Gate;
use crate::core::pocket::Pocket;
use crate::economy::budget::Budget;
use crate::economy::tray::BallEconomy;
use crate::kb::facts::{Fact, KnowledgeBase};
use crate::state::machine::State;

pub struct InferenceEngine {
    pipeline: Pipeline,
    fixer: TokenFixer,
    config: InferenceConfig,
}

impl InferenceEngine {
    pub fn new() -> Self {
        InferenceEngine {
            pipeline: Pipeline::new(),
            fixer: TokenFixer::new(),
            config: InferenceConfig::new(),
        }
    }

    pub fn with_config(config: InferenceConfig) -> Self {
        let budget = Budget::new(config.budget_tokens, config.budget_cost_usd);
        InferenceEngine {
            pipeline: Pipeline::new().with_budget(budget),
            fixer: TokenFixer::new(),
            config,
        }
    }

    pub fn validate(&mut self, request: ValidationRequest) -> CidResult<ValidationResult> {
        let domain = request.domain.as_deref().or(self.config.domain.as_deref());
        let mut result = self.pipeline.validate(&request, domain)?;

        if self.config.auto_fix && !result.passed {
            let (fixed_text, fixes) = self.fixer.fix(&result.validated_text, &request.context);
            if !fixes.is_empty() {
                let re_request = ValidationRequest::new(&fixed_text, &request.context)
                    .with_domain(request.domain.as_deref().unwrap_or("general"));

                let re_result = self.pipeline.validate(&re_request, domain)?;
                result.validated_text = re_result.validated_text;
                result.fixes = fixes;
                result.confidence = re_result.confidence;
                result.passed = re_result.passed;
                result.gate_scores = re_result.gate_scores;
            }
        }

        Ok(result)
    }

    pub fn validate_token(&mut self, token: &str, context: &str) -> CidResult<Ball> {
        self.pipeline.validate_single_token(token, context, token)
    }

    pub fn validate_beam(
        &mut self,
        candidates: Vec<String>,
        context: &str,
    ) -> CidResult<Option<Pocket>> {
        let claim = candidates.join(" ");
        let balls = self
            .pipeline
            .validate_candidates(&candidates, context, &claim)?;
        Ok(self.pipeline.select_best(balls))
    }

    pub fn fix(&mut self, text: &str, context: &str) -> (String, Vec<TokenFix>) {
        self.fixer.fix(text, context)
    }

    pub fn validate_batch(
        &mut self,
        requests: Vec<ValidationRequest>,
    ) -> Vec<CidResult<ValidationResult>> {
        requests.into_iter().map(|r| self.validate(r)).collect()
    }

    pub fn set_domain(&mut self, domain: &str) {
        self.config.domain = Some(domain.to_string());
    }

    pub fn set_strict(&mut self, strict: bool) {
        self.config.strict_mode = strict;
    }

    pub fn adjust_threshold(&mut self, gate: Gate, threshold: f64) {
        self.pipeline.pin_field.adjust_pin(gate, threshold);
    }

    /// Activate the two hallucination-hardening gates on this engine's pipeline:
    /// `DomainBinding` (refuse tokens lacking provenance in the verified context)
    /// and `ProofRecompute` (refuse claims whose proof does not recompute). Used
    /// by the LLM completion path so generated text is gated before emission.
    pub fn enable_domain_proof_gates(&mut self) {
        self.pipeline.pin_field = crate::core::pin::PinField::with_domain_proof_enforcement();
    }

    pub fn state(&self) -> &State {
        self.pipeline.state()
    }

    pub fn economy(&self) -> &BallEconomy {
        self.pipeline.economy()
    }

    pub fn budget(&self) -> &Budget {
        self.pipeline.budget()
    }

    pub fn reset(&mut self) {
        self.pipeline.reset();
    }

    pub fn add_fact(&mut self, fact: Fact) {
        self.pipeline.kb.add_fact(fact);
    }

    pub fn lookup_fact(&self, name: &str) -> Option<&Fact> {
        self.pipeline.kb.lookup(name)
    }

    pub fn search_facts(&self, query: &str) -> Vec<&Fact> {
        self.pipeline.kb.search(query)
    }

    pub fn kb_count(&self) -> usize {
        self.pipeline.kb.count()
    }

    pub fn detect_fallacies(&self, text: &str) -> Vec<crate::gates::fallacy::FallacyMatch> {
        crate::gates::fallacy::FallacyGate::new().detect(text)
    }

    pub fn fallacy_score(&self, text: &str) -> f64 {
        crate::gates::fallacy::FallacyGate::new().score(text)
    }

    pub fn sanity_check(&self, value: f64, category: &str) -> Option<sanity::SanityResult> {
        sanity::SanityChecker::new().check(value, category)
    }

    pub fn sanity_score(&self, value: f64, category: &str) -> f64 {
        sanity::SanityChecker::new().score(value, category)
    }

    pub fn detect_biases(&self, text: &str) -> Vec<bias::BiasMatch> {
        bias::BiasDetector::new().detect(text)
    }

    pub fn bias_score(&self, text: &str) -> f64 {
        bias::BiasDetector::new().score(text)
    }

    pub fn set_kb(&mut self, kb: KnowledgeBase) {
        self.pipeline.kb = kb;
    }

    pub fn kb(&self) -> &KnowledgeBase {
        &self.pipeline.kb
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = InferenceEngine::new();
        assert_eq!(engine.state(), &State::Normal);
        assert!(engine.budget().remaining_tokens() > 0);
    }

    #[test]
    fn test_validate_token() {
        let mut engine = InferenceEngine::new();
        let ball = engine.validate_token("42", "number").unwrap();
        assert!(ball.all_passed());
    }

    #[test]
    fn test_validate_beam() {
        let mut engine = InferenceEngine::new();
        let candidates = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let pocket = engine.validate_beam(candidates, "number").unwrap();
        assert!(pocket.is_some());
    }

    #[test]
    fn test_fix() {
        let mut engine = InferenceEngine::new();
        let (fixed, fixes) = engine.fix("2 + 3 = 6", "math");
        assert_eq!(fixed, "2 + 3 = 5");
        assert_eq!(fixes.len(), 1);
    }

    #[test]
    fn test_knowledge_base() {
        let engine = InferenceEngine::new();
        assert!(engine.lookup_fact("pi").is_some());
        assert!(engine.kb_count() > 100);
    }
}
