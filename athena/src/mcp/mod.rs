mod compact;
mod server;
mod tools;

pub use compact::{DEFAULT_LIMIT, Detail, SavingsLedger, compact_data, estimate_tokens};
pub use server::McpHandler;
pub use tools::{AthenaTool, ToolRegistry};

use crate::astrology::Aspect;
use crate::bankai::Bankai;
#[cfg(feature = "budget")]
use crate::budget::TokenBudget;
use crate::entity::EntityRegistry;
use crate::formula::FormulaRegistry;
use crate::gates::confidence::ConfidenceGate;
use crate::gates::fact::FactGate;
use crate::gates::formal::FormalGate;
use crate::gates::logic::LogicGate;
use crate::gates::math::MathGate;
use crate::gates::{Gate, GateResult};
use crate::wheel::Domain;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct AthenaRequest {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AthenaResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

impl AthenaResponse {
    pub fn ok(data: serde_json::Value) -> Self {
        AthenaResponse {
            success: true,
            data,
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        AthenaResponse {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error),
        }
    }
}

pub struct AthenaMCP {
    pub engine: Bankai,
    pub tools: ToolRegistry,
    pub entities: EntityRegistry,
    /// Context-token savings from response compaction (recorded by the
    /// MCP wire layer in `server.rs`, reported by the `savings` method).
    pub savings: Mutex<SavingsLedger>,
    #[cfg(feature = "budget")]
    pub budget: Mutex<TokenBudget>,
}

impl AthenaMCP {
    // ─── Parameter extraction helpers ──────────────────────────────
    /// Extract a string parameter with a default fallback.
    fn str_param(&self, params: &serde_json::Value, key: &str, default: &str) -> String {
        params
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    /// Extract an optional string parameter.
    fn opt_str_param<'a>(&self, params: &'a serde_json::Value, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Extract a u64 parameter with a default fallback.
    fn u64_param(&self, params: &serde_json::Value, key: &str, default: u64) -> u64 {
        params.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    /// Extract a bool parameter with a default fallback.
    fn bool_param(&self, params: &serde_json::Value, key: &str, default: bool) -> bool {
        params.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    /// Parse args from a JSON object parameter.
    fn parse_args(&self, params: &serde_json::Value, key: &str) -> HashMap<String, f64> {
        let mut args = HashMap::new();
        if let Some(obj) = params.get(key).and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(n) = v.as_f64() {
                    args.insert(k.clone(), n);
                }
            }
        }
        args
    }

    pub fn with_entities(registry: FormulaRegistry, entities: EntityRegistry) -> Self {
        let engine = Bankai::with_entities(registry.clone(), entities.clone());
        let tools = ToolRegistry::new();
        AthenaMCP {
            engine,
            tools,
            entities,
            savings: Mutex::new(SavingsLedger::default()),
            #[cfg(feature = "budget")]
            budget: Mutex::new(TokenBudget::default()),
        }
    }

    /// Like `with_entities`, but with an explicit token budget.
    #[cfg(feature = "budget")]
    pub fn with_context(
        registry: FormulaRegistry,
        entities: EntityRegistry,
        budget: TokenBudget,
    ) -> Self {
        let mut mcp = Self::with_entities(registry, entities);
        mcp.budget = Mutex::new(budget);
        mcp
    }

    pub fn handle_request(&self, request: AthenaRequest) -> AthenaResponse {
        match request.method.as_str() {
            "ping" => AthenaResponse::ok(serde_json::json!({"pong": true})),

            "savings" => {
                let ledger = self
                    .savings
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                AthenaResponse::ok(serde_json::json!({
                    "calls": ledger.calls,
                    "baseline_tokens": ledger.baseline_tokens,
                    "emitted_tokens": ledger.emitted_tokens,
                    "saved_tokens": ledger.saved(),
                    "saved_pct": ledger.saved_pct(),
                }))
            }

            #[cfg(feature = "budget")]
            "budget_stats" => {
                let budget = self
                    .budget
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                AthenaResponse::ok(serde_json::to_value(budget.stats()).unwrap_or_default())
            }

            #[cfg(feature = "budget")]
            "budget_reset" => {
                let mut budget = self
                    .budget
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                budget.reset();
                AthenaResponse::ok(serde_json::json!({"reset": true}))
            }

            "tools" => {
                let tool_list: Vec<serde_json::Value> = self
                    .tools
                    .all()
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name(),
                            "description": t.description(),
                            "parameters": t.parameters(),
                        })
                    })
                    .collect();
                AthenaResponse::ok(serde_json::json!({"tools": tool_list}))
            }

            "validate" => {
                let text = request
                    .params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let gate = request
                    .params
                    .get("gate")
                    .and_then(|v| v.as_str())
                    .unwrap_or("math");
                let result = self.run_validation(text, gate);
                AthenaResponse::ok(serde_json::to_value(result).unwrap_or_default())
            }

            "traverse" => {
                let domain_str = request
                    .params
                    .get("domain")
                    .and_then(|v| v.as_str())
                    .unwrap_or("surya");
                let max_depth = request
                    .params
                    .get("max_depth")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                let domain = match Domain::parse(domain_str) {
                    Some(d) => d,
                    None => {
                        return AthenaResponse::err(format!(
                            "unknown domain '{}'. Valid: surya, chandra, mangala, budha, brihaspati, shukra, shani, rahu, ketu",
                            domain_str
                        ));
                    }
                };
                let traversal = self.engine.traverse(domain, max_depth);
                AthenaResponse::ok(serde_json::json!({
                    "start": domain_str,
                    "formulas_found": traversal.formula_count(),
                    "domains_visited": traversal.domains_visited().iter().map(|d| d.full_name()).collect::<Vec<_>>(),
                }))
            }

            "compose" => {
                let formulas: Vec<&str> = request
                    .params
                    .get("formulas")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                match self.engine.compose(&formulas) {
                    Ok(comp) => AthenaResponse::ok(serde_json::json!({
                        "confidence": comp.confidence,
                        "formulas": comp.formulas.iter().map(|f| f.id.clone()).collect::<Vec<_>>(),
                    })),
                    Err(e) => AthenaResponse::err(e.to_string()),
                }
            }

            "formula_search" => {
                let keyword = request
                    .params
                    .get("keyword")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let results = self.engine.formulas.search(keyword);
                AthenaResponse::ok(serde_json::json!({
                    "count": results.len(),
                    "formulas": results.iter().map(|f| serde_json::json!({
                        "id": f.id,
                        "domain": f.domain.full_name(),
                        "type": format!("{:?}", f.formula_type),
                        "description": f.description,
                    })).collect::<Vec<_>>(),
                }))
            }

            "formula_by_output" => {
                let output = request
                    .params
                    .get("output")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let results = self.engine.formulas.by_output(output);
                AthenaResponse::ok(serde_json::json!({
                    "count": results.len(),
                    "formulas": results.iter().map(|f| serde_json::json!({
                        "id": f.id,
                        "domain": f.domain.full_name(),
                        "type": format!("{:?}", f.formula_type),
                        "description": f.description,
                        "inputs": f.inputs,
                        "output": f.output,
                        "expression": f.expression,
                    })).collect::<Vec<_>>(),
                }))
            }

            "evaluate" => self.handle_evaluate(&request.params),

            "wheel" => {
                let domain_str = self.opt_str_param(&request.params, "domain");
                let wheel = crate::wheel::WheelGraph::new();
                if let Some(ds) = domain_str {
                    if let Some(parsed) = Domain::parse(ds) {
                        let node = wheel.node(parsed);
                        AthenaResponse::ok(serde_json::json!({
                            "domain": ds,
                            "symbol": node.symbol,
                            "name": node.name,
                            "description": node.description,
                            "knowledge_domain": parsed.knowledge_domain(),
                            "opposite": node.opposite.full_name(),
                            "trines": parsed.trines().iter().map(|t| t.full_name()).collect::<Vec<_>>(),
                            "adjacent": parsed.adjacent().iter().map(|a| a.full_name()).collect::<Vec<_>>(),
                        }))
                    } else {
                        AthenaResponse::err(format!("Unknown domain: '{}'", ds))
                    }
                } else {
                    let all: Vec<serde_json::Value> = wheel
                        .all_nodes()
                        .iter()
                        .map(|node| {
                            serde_json::json!({
                                "symbol": node.symbol,
                                "name": node.name,
                                "description": node.description,
                                "opposite": node.opposite.full_name(),
                            })
                        })
                        .collect();
                    AthenaResponse::ok(serde_json::json!({"domains": all, "count": all.len()}))
                }
            }

            "reason" => self.handle_reason(&request.params),

            "classify" => self.handle_classify(&request.params),

            "ephemeris" => {
                let date = request
                    .params
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let time = request
                    .params
                    .get("time")
                    .and_then(|v| v.as_str())
                    .unwrap_or("00:00");
                let dparts: Vec<i32> = date.split('-').filter_map(|p| p.parse().ok()).collect();
                let tparts: Vec<f64> = time.split(':').filter_map(|p| p.parse().ok()).collect();
                if dparts.len() != 3 || tparts.is_empty() {
                    return AthenaResponse::err(
                        "expected date as YYYY-MM-DD and time as HH:MM".to_string(),
                    );
                }
                let hour = tparts[0] + tparts.get(1).copied().unwrap_or(0.0) / 60.0;
                let jd = crate::ephemeris::julian_day(
                    dparts[0] as i16,
                    dparts[1] as u8,
                    dparts[2] as u8,
                    hour,
                );
                // 4 decimal places (~0.4 arcsecond) keeps responses token-lean
                // without losing placement-relevant precision.
                let round4 = |x: f64| (x * 10000.0).round() / 10000.0;
                let positions: Vec<crate::ephemeris::GrahaPosition> = match request
                    .params
                    .get("graha")
                    .and_then(|v| v.as_str())
                {
                    Some(g) => match crate::astrology::Graha::parse(g) {
                        Some(graha) => vec![crate::ephemeris::graha_position(graha, jd)],
                        None => {
                            return AthenaResponse::err(format!(
                                "unknown graha '{g}'. Valid: surya, chandra, mangala, budha, brihaspati, shukra, shani, rahu, ketu"
                            ));
                        }
                    },
                    None => crate::ephemeris::all_graha_positions(jd),
                };
                AthenaResponse::ok(serde_json::json!({
                    "jd": jd,
                    "ayanamsa": round4(crate::ephemeris::lahiri_ayanamsa(jd)),
                    "positions": positions.iter().map(|p| serde_json::json!({
                        "graha": p.graha.full_name(),
                        "tropical": round4(p.tropical),
                        "sidereal": round4(p.sidereal),
                        "rashi": p.rashi.name(),
                        "nakshatra": p.nakshatra.name(),
                        "pada": p.pada,
                    })).collect::<Vec<_>>(),
                }))
            }

            "gyro" => self.handle_gyro(),

            "entity_list" => self.handle_entity_list(),

            "entity_get" => self.handle_entity_get(&request.params),

            "entity_aspect" => self.handle_entity_aspect(&request.params),

            "entity_search" => self.handle_entity_search(&request.params),

            "entity_eval" => {
                let formula_id = request
                    .params
                    .get("formula")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let entity_id = request
                    .params
                    .get("entity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let args_obj = request.params.get("args").and_then(|v| v.as_object());

                // Seed entities have properties/constants for formula evaluation
                let seed = match self.entities.get_seed(entity_id) {
                    Some(s) => s,
                    None => {
                        return AthenaResponse::err(format!(
                            "Seed entity not found (entity_eval requires a seed entity): '{}'",
                            entity_id
                        ));
                    }
                };
                let mut args = HashMap::new();
                if let Some(obj) = args_obj {
                    for (k, v) in obj {
                        if let Some(n) = v.as_f64() {
                            args.insert(k.clone(), n);
                        }
                    }
                }
                if let Some(f) = self.engine.formulas.get(formula_id) {
                    let entity_sign = seed
                        .classification
                        .as_ref()
                        .and_then(|c| c.dominant_sign())
                        .map(crate::wheel::Domain::from_sign);
                    let domain_aligned = entity_sign == Some(f.domain);
                    for input in &f.inputs {
                        if !args.contains_key(input) {
                            let val = seed
                                .properties
                                .get(input)
                                .cloned()
                                .or_else(|| Bankai::match_constant(&seed.constants, input));
                            if let Some(v) = val {
                                args.insert(input.clone(), v);
                            }
                        }
                    }
                    match self.engine.evaluate(formula_id, &args) {
                        Ok(value) => AthenaResponse::ok(serde_json::json!({
                            "formula": formula_id,
                            "entity": entity_id,
                            "entity_name": seed.name,
                            "domain_aligned": domain_aligned,
                            "result": value,
                            "args": args,
                        })),
                        Err(e) => AthenaResponse::err(format!("Evaluation failed: {}", e)),
                    }
                } else {
                    AthenaResponse::err(format!("Formula not found: '{}'", formula_id))
                }
            }

            _ => AthenaResponse::err(format!("Unknown method: {}", request.method)),
        }
    }

    fn run_validation(&self, text: &str, gate: &str) -> GateResult {
        let gates: Vec<Box<dyn Gate>> = match gate {
            "math" => vec![Box::new(MathGate::new())],
            "logic" => vec![Box::new(LogicGate::new())],
            "confidence" => vec![Box::new(ConfidenceGate::new())],
            "formal" => vec![Box::new(FormalGate::new())],
            "fact" => vec![Box::new(FactGate::new(None))],
            "all" => vec![
                Box::new(MathGate::new()),
                Box::new(FormalGate::new()),
                Box::new(LogicGate::new()),
                Box::new(ConfidenceGate::new()),
            ],
            _ => vec![Box::new(MathGate::new())],
        };
        GateResult::run(gates, text)
    }

    // ─── Extracted handler methods ──────────────────────────────

    fn handle_classify(&self, params: &serde_json::Value) -> AthenaResponse {
        let text = self.str_param(params, "text", "");
        if text.is_empty() {
            return AthenaResponse::err("No text provided for classification.".to_string());
        }
        let sorter = crate::astrology::ChangeSorter::new();
        let classification = sorter.classify_token(&text);
        let dominant_sign = classification.dominant_sign();
        let dominant_element = classification.dominant_element();
        let dominant_modality = classification.dominant_modality();
        let dominant_graha = classification.vedic.dominant_graha();
        let dominant_guna = classification.vedic.dominant_guna();
        let dominant_nakshatra = classification.vedic.dominant_nakshatra();
        let dominant_vedic_element = classification.vedic.dominant_vedic_element();
        AthenaResponse::ok(serde_json::json!({
            "text": text,
            "classification": {
                "signs": classification.signs,
                "elements": classification.elements,
                "modalities": classification.modalities,
                "rulers": classification.rulers,
                "houses": classification.houses,
                "aspects": classification.aspects,
                "polarity": classification.polarity,
            },
            "dominant": {
                "sign": dominant_sign.map(|s| format!("{:?}", s)),
                "element": dominant_element.map(|e| format!("{:?}", e)),
                "modality": dominant_modality.map(|m| format!("{:?}", m)),
                "graha": dominant_graha.map(|g| g.name()),
                "guna": dominant_guna.map(|g| g.name()),
                "nakshatra": dominant_nakshatra.map(|n| format!("{:?}", n)),
                "vedic_element": dominant_vedic_element.map(|ve| ve.sanskrit()),
            },
            "vedic": {
                "grahas": classification.vedic.grahas,
                "gunas": classification.vedic.gunas,
                "vedic_elements": classification.vedic.vedic_elements,
                "confidence": classification.vedic.confidence,
            }
        }))
    }

    fn handle_gyro(&self) -> AthenaResponse {
        let gyro = self.engine.gyro_state();
        let (dominant_sign, dominant_mass) = gyro.dominant_sign_info();
        let weights = gyro.alignment_weights();
        AthenaResponse::ok(serde_json::json!({
            "orientation": gyro.orientation.0,
            "dominant_sign": format!("{:?}", dominant_sign),
            "dominant_mass": dominant_mass,
            "angular_velocity": gyro.angular_velocity,
            "precession": gyro.precession(),
            "mass_distribution": gyro.mass_distribution,
            "alignment_weights": weights,
        }))
    }

    fn handle_entity_list(&self) -> AthenaResponse {
        let seeds: Vec<serde_json::Value> = self
            .entities
            .list_seeds()
            .iter()
            .filter_map(|id| self.entities.get_seed(id))
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "kind": "seed",
                    "description": s.description,
                    "tags": s.tags,
                    "properties": s.properties,
                })
            })
            .collect();
        let runtime: Vec<serde_json::Value> = self
            .entities
            .list()
            .iter()
            .filter_map(|id| self.entities.get(id))
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "text": e.text,
                    "kind": "runtime",
                    "dominant_sign": e.dominant_sign().map(|s| format!("{:?}", s)),
                    "tags": e.tags,
                    "seq": e.seq,
                })
            })
            .collect();
        let all: Vec<serde_json::Value> = seeds.into_iter().chain(runtime).collect();
        AthenaResponse::ok(serde_json::json!({"count": all.len(), "entities": all}))
    }

    fn handle_entity_aspect(&self, params: &serde_json::Value) -> AthenaResponse {
        let from = self.str_param(params, "from", "");
        let to = self.str_param(params, "to", "");

        // Try runtime entities first.
        if let Some((aspect, a, b)) = self.entities.aspect_between(&from, &to) {
            let sign_a = a
                .dominant_sign()
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "unknown".to_string());
            let sign_b = b
                .dominant_sign()
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "unknown".to_string());
            return AthenaResponse::ok(serde_json::json!({
                "entity_a": a.text,
                "entity_b": b.text,
                "sign_a": sign_a,
                "sign_b": sign_b,
                "aspect": format!("{:?}", aspect),
                "aspect_quality": aspect.quality(),
                "arc_distance": Aspect::arc_distance_between(
                    a.dominant_sign().map_or(0, |s| s.index()),
                    b.dominant_sign().map_or(0, |s| s.index()),
                ),
            }));
        }

        // Fall back to seed entities.
        let seed_a = self.entities.get_seed(&from);
        let seed_b = self.entities.get_seed(&to);
        if let (Some(sa), Some(sb)) = (seed_a, seed_b) {
            let sign_from = |s: &crate::entity::SeedEntity| -> Option<crate::astrology::Sign> {
                s.rashi.as_ref().and_then(|r| {
                    let r_lower = r.to_lowercase();
                    match r_lower.as_str() {
                        "mesha" => Some(crate::astrology::Sign::Aries),
                        "vrishabha" => Some(crate::astrology::Sign::Taurus),
                        "mithuna" => Some(crate::astrology::Sign::Gemini),
                        "karka" => Some(crate::astrology::Sign::Cancer),
                        "simha" => Some(crate::astrology::Sign::Leo),
                        "kanya" => Some(crate::astrology::Sign::Virgo),
                        "tula" => Some(crate::astrology::Sign::Libra),
                        "vrishchika" => Some(crate::astrology::Sign::Scorpio),
                        "dhanu" => Some(crate::astrology::Sign::Sagittarius),
                        "makara" => Some(crate::astrology::Sign::Capricorn),
                        "kumbha" => Some(crate::astrology::Sign::Aquarius),
                        "meena" => Some(crate::astrology::Sign::Pisces),
                        _ => None,
                    }
                })
            };
            let s_a = sign_from(sa);
            let s_b = sign_from(sb);
            if let (Some(sa_sign), Some(sb_sign)) = (s_a, s_b)
                && let Some(aspect) =
                    crate::astrology::Aspect::between_sign_indices(sa_sign.index(), sb_sign.index())
            {
                return AthenaResponse::ok(serde_json::json!({
                    "entity_a": sa.name,
                    "entity_b": sb.name,
                    "sign_a": format!("{:?}", sa_sign),
                    "sign_b": format!("{:?}", sb_sign),
                    "aspect": format!("{:?}", aspect),
                    "aspect_quality": aspect.quality(),
                    "arc_distance": Aspect::arc_distance_between(sa_sign.index(), sb_sign.index()),
                }));
            }
            let missing = if s_a.is_none() { &from } else { &to };
            return AthenaResponse::err(format!(
                "Cannot compute aspect: seed entity '{}' has no rashi (dominant sign) set",
                missing
            ));
        }

        AthenaResponse::err(format!(
            "Cannot compute aspect: '{}' or '{}' not found in runtime or seed registries",
            from, to
        ))
    }

    fn handle_entity_get(&self, params: &serde_json::Value) -> AthenaResponse {
        let id = self.str_param(params, "id", "");
        if let Some(s) = self.entities.get_seed(&id) {
            let mut response = serde_json::json!({
                "entity": {
                    "id": s.id,
                    "name": s.name,
                    "kind": "seed",
                    "description": s.description,
                    "tags": s.tags,
                    "properties": s.properties,
                    "constants": s.constants,
                    "formula": s.formula,
                }
            });
            if let Some(g) = s.dominant_graha() {
                response["entity"]["graha"] = serde_json::json!(g.full_name());
            }
            if let Some(g) = s.dominant_guna() {
                response["entity"]["guna"] = serde_json::json!(g.name());
            }
            if let Some(ref m) = s.mantra {
                response["entity"]["mantra"] = serde_json::json!(m);
            }
            if let Some(ref b) = s.bija {
                response["entity"]["bija"] = serde_json::json!(b);
            }
            if let Some(ref d) = s.day {
                response["entity"]["day"] = serde_json::json!(d);
            }
            if !s.ruled_nakshatras.is_empty() {
                response["entity"]["ruled_nakshatras"] = serde_json::json!(s.ruled_nakshatras);
            }
            AthenaResponse::ok(response)
        } else if let Some(e) = self.entities.get(&id) {
            let mut response = serde_json::json!({
                "entity": {
                    "id": e.id,
                    "text": e.text,
                    "kind": "runtime",
                    "dominant_sign": e.dominant_sign().map(|s| format!("{:?}", s)),
                    "dominant_element": e.dominant_element().map(|el| format!("{:?}", el)),
                    "tags": e.tags,
                    "values": e.values,
                    "truth": e.truth,
                    "seq": e.seq,
                }
            });
            if let Some(g) = e.dominant_graha() {
                response["entity"]["dominant_graha"] = serde_json::json!(g.full_name());
            }
            if let Some(g) = e.dominant_guna() {
                response["entity"]["dominant_guna"] = serde_json::json!(g.name());
            }
            if let Some(n) = e.dominant_nakshatra() {
                response["entity"]["dominant_nakshatra"] = serde_json::json!(format!("{:?}", n));
            }
            if let Some(ve) = e.dominant_vedic_element() {
                response["entity"]["dominant_vedic_element"] = serde_json::json!(ve.name());
            }
            AthenaResponse::ok(response)
        } else {
            AthenaResponse::err(format!("Entity not found: '{}'", id))
        }
    }

    fn handle_entity_search(&self, params: &serde_json::Value) -> AthenaResponse {
        let keyword = self.str_param(params, "keyword", "");
        let seed_results = self.entities.search_seeds(&keyword);
        let runtime_results = self.entities.search(&keyword);
        let mut entities: Vec<serde_json::Value> = Vec::new();
        for s in &seed_results {
            let mut entry = serde_json::json!({
                "id": s.id,
                "name": s.name,
                "kind": "seed",
                "description": s.description,
            });
            if let Some(g) = s.dominant_graha() {
                entry["graha"] = serde_json::json!(g.full_name());
            }
            if let Some(g) = s.dominant_guna() {
                entry["guna"] = serde_json::json!(g.name());
            }
            entities.push(entry);
        }
        for e in &runtime_results {
            let mut entry = serde_json::json!({
                "id": e.id,
                "text": e.text,
                "kind": "runtime",
                "dominant_sign": e.dominant_sign().map(|s| format!("{:?}", s)),
            });
            if let Some(g) = e.dominant_graha() {
                entry["dominant_graha"] = serde_json::json!(g.full_name());
            }
            entities.push(entry);
        }
        AthenaResponse::ok(serde_json::json!({"count": entities.len(), "entities": entities}))
    }

    fn handle_evaluate(&self, params: &serde_json::Value) -> AthenaResponse {
        let formula_id = self.str_param(params, "formula", "");
        let args = self.parse_args(params, "args");
        match self.engine.evaluate(&formula_id, &args) {
            Ok(value) => AthenaResponse::ok(serde_json::json!({
                "formula": formula_id,
                "result": value,
            })),
            Err(e) => AthenaResponse::err(format!("Evaluation failed: {}", e)),
        }
    }

    fn handle_reason(&self, params: &serde_json::Value) -> AthenaResponse {
        let have_str = self.str_param(params, "have", "");
        let want = self.str_param(params, "want", "");
        let max_depth = self.u64_param(params, "max_depth", 5) as usize;
        let execute = self.bool_param(params, "execute", false);

        let have_vars: Vec<String> = have_str.split(',').map(|s| s.trim().to_string()).collect();
        match self.engine.find_path(&have_vars, &want, max_depth) {
            Ok(path) => {
                if path.is_empty() {
                    return AthenaResponse::ok(serde_json::json!({
                        "already_have": true,
                        "message": format!("Already have '{}' — no derivation needed.", want),
                    }));
                }
                let formulas_info: Vec<serde_json::Value> = path
                    .iter()
                    .filter_map(|fid| {
                        self.engine.formulas.get(fid).map(|f| {
                            serde_json::json!({
                                "id": f.id,
                                "domain": f.domain.full_name(),
                                "inputs": f.inputs,
                                "output": f.output,
                            })
                        })
                    })
                    .collect();

                let mut result = serde_json::json!({
                    "have": have_vars,
                    "want": want,
                    "formula_chain": formulas_info,
                });

                if execute {
                    let args = self.parse_args(params, "args");
                    let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                    match self.engine.chain(&refs, &args) {
                        Ok(chain_result) => {
                            result["execution"] = serde_json::json!(chain_result);
                        }
                        Err(e) => {
                            result["execution_error"] = serde_json::json!(e.to_string());
                        }
                    }
                }

                AthenaResponse::ok(result)
            }
            Err(e) => AthenaResponse::err(e.to_string()),
        }
    }
}
