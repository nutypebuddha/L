use crate::inference::json::{stringify, JsonValue};
use crate::inference::request::ValidationRequest;
use crate::inference::InferenceEngine;

pub fn list_tools() -> JsonValue {
    JsonValue::Array(vec![
        make_tool(
            "cid_validate",
            "Validate text through CID's validation gates (math, logic, fact, confidence). Returns validated text, confidence score, and whether it passed.",
            &[
                ("text", "string", "Text to validate", true),
                (
                    "context",
                    "string",
                    "Context domain (e.g. 'math', 'facts', 'logic', 'english')",
                    true,
                ),
                (
                    "domain",
                    "string",
                    "Optional domain override for confidence thresholds",
                    false,
                ),
            ],
        ),
        make_tool(
            "cid_fix",
            "Auto-fix errors in text: wrong math sums/products, common typos, code consistency (var->let in JS). Returns fixed text and list of fixes applied.",
            &[
                ("text", "string", "Text to fix", true),
                (
                    "context",
                    "string",
                    "Context domain (e.g. 'math', 'javascript', 'english')",
                    true,
                ),
            ],
        ),
        make_tool(
            "cid_lookup",
            "Look up a known fact by name in CID's knowledge base (pi, e, c, g, G, h, kB, water_density, air_density, speed_sound, earth_mass, sun_mass, earth_radius, au, light_year).",
            &[("name", "string", "Fact name to look up", true)],
        ),
        make_tool(
            "cid_search",
            "Search CID's knowledge base by keyword. Returns all matching facts with their values and units.",
            &[(
                "query",
                "string",
                "Search query (e.g. 'speed', 'mass', 'density')",
                true,
            )],
        ),
        make_tool(
            "cid_detect_fallacies",
            "Detect logical fallacies in text using pattern matching. Identifies ad hominem, straw man, bandwagon, slippery slope, false dilemma, circular reasoning, appeal to emotion, sunk cost, red herring, false cause, and more. Returns list of detected fallacies with confidence scores.",
            &[("text", "string", "Text to analyze for fallacies", true)],
        ),
        make_tool(
            "cid_sanity_check",
            "Check if a numeric value is within reasonable range for a given physical category (e.g. speed, temperature, height, weight, energy, power, distance, time, price, percent). Returns whether value is in typical range and valid range.",
            &[
                ("value", "number", "Numeric value to check", true),
                (
                    "category",
                    "string",
                    "Physical category (e.g. 'speed_mph', 'temp_c', 'height_m', 'weight_kg', 'price_usd')",
                    true,
                ),
            ],
        ),
        make_tool(
            "cid_detect_biases",
            "Detect cognitive biases in text: anchoring, availability, confirmation, sunk cost, bandwagon, framing, hindsight, Dunning-Kruger, recency, negativity, authority bias, automation bias. Returns list of detected biases with mitigations.",
            &[(
                "text",
                "string",
                "Text to analyze for cognitive biases",
                true,
            )],
        ),
        make_tool(
            "cid_sample",
            "Request an LLM completion through the MCP client. CID acts as a sampling proxy, allowing the client to request completions that CID will validate. Returns the validated completion.",
            &[
                ("prompt", "string", "Prompt to send to the LLM", true),
                (
                    "max_tokens",
                    "number",
                    "Maximum tokens to generate (default: 1000)",
                    false,
                ),
            ],
        ),
        // === DYNAMIC KB TOOLS (Phase 1) ===
        make_tool(
            "cid_kb_add",
            "Add a fact to CID's knowledge base dynamically. Facts persist for the session, tagged with domain and confidence. Returns the created fact.",
            &[
                ("name", "string", "Fact name (e.g. 'my_custom_value')", true),
                ("value", "number", "Numeric value", true),
                ("unit", "string", "Unit string (e.g. 'kg', 'USD', '')", true),
                ("source", "string", "Source description", true),
                (
                    "domain",
                    "string",
                    "Optional domain (alpha..mu, default: alpha)",
                    false,
                ),
                (
                    "confidence",
                    "number",
                    "Optional confidence 0.0-1.0 (default: 0.8)",
                    false,
                ),
            ],
        ),
        make_tool(
            "cid_kb_context",
            "Context-aware knowledge base retrieval. Returns facts ranked by relevance to query+domain, using keyword match + recency + confidence scoring.",
            &[
                ("query", "string", "Search query", true),
                (
                    "domain",
                    "string",
                    "Domain (alpha..mu) to search within",
                    true,
                ),
                (
                    "max_results",
                    "number",
                    "Maximum results (default: 10)",
                    false,
                ),
            ],
        ),
        make_tool(
            "cid_kb_recent",
            "Get facts added within a recent time window. Shows dynamic and session facts.",
            &[(
                "within_secs",
                "number",
                "Time window in seconds (default: 3600)",
                false,
            )],
        ),
        // === TANTO MERGED TOOLS ===
        make_tool(
            "cid_tanto_rational",
            "Evaluate an expression as an exact rational (fraction). Returns the result as a fraction string like '1/3' instead of '0.3333...'. Only handles integers and the operators + - * / ( ). Falls back to f64 for non-rational expressions (decimals, trig). Ideal for chained fraction arithmetic without floating-point drift.",
            &[(
                "expression",
                "string",
                "Rational expression to evaluate (e.g. '1/3 + 1/6')",
                true,
            )],
        ),
        make_tool(
            "cid_tanto_eval",
            "Evaluate a math expression using Tanto's compute engine. Supports full expressions (2+3, sqrt(144), 15% of 240), named ops (add 2 3, avg 12 15 18), constants (pi, e, c, g, G, h, kB), and natural language. Returns the computed result.",
            &[("expression", "string", "Math expression to evaluate", true)],
        ),
        make_tool(
            "cid_tanto_convert",
            "Convert values between units using Tanto's conversion engine. Supports 60+ conversions across length (mi, km, m, ft), speed (mph, kmh, ms, knot), temperature (F, C, K), weight (lb, kg), volume (gal, L), data (MB, GB, KB, B), and time (hr, min).",
            &[
                ("value", "number", "Numeric value to convert", true),
                (
                    "from",
                    "string",
                    "Source unit (e.g. mph, mi, F, lb, gal)",
                    true,
                ),
                ("to", "string", "Target unit (e.g. kmh, km, C, kg, L)", true),
            ],
        ),
        make_tool(
            "cid_tanto_formula",
            "Compute a verified physics/geometry/finance formula. 22 formulas available: circle_area, sphere_volume, ke (kinetic energy), pe (potential energy), ohm_v, ohm_i, momentum, force, pressure, work, power, compound_amount, and more.",
            &[
                (
                    "name",
                    "string",
                    "Formula name (e.g. circle_area, ke, pe, ohm_v)",
                    true,
                ),
                ("args", "string", "Space-separated numeric arguments", true),
            ],
        ),
        make_tool(
            "cid_tanto_solve",
            "Run a multi-step solver template. 9 solvers: orbit (orbital mechanics), projectile (trajectory), energy (E=mc^2), fall (free fall), ke (kinetic energy), pe (potential energy), ohm (Ohm's law), compound (interest), growth (exponential). Returns detailed step-by-step solution.",
            &[
                (
                    "solver",
                    "string",
                    "Solver name (orbit, projectile, energy, fall, ke, pe, ohm, compound, growth)",
                    true,
                ),
                (
                    "args",
                    "string",
                    "Space-separated numeric arguments for the solver",
                    true,
                ),
            ],
        ),
        make_tool(
            "cid_tanto_think",
            "Apply a structured thinking framework to a problem. 6 frameworks: ooda (Observe-Orient-Decide-Act), swot (Strengths-Weaknesses-Opportunities-Threats), cynefin (domain classification), why5 (root cause analysis), firstprinciples (axiomatic reasoning), shuhari (stages of mastery).",
            &[
                (
                    "framework",
                    "string",
                    "Thinking framework (ooda, swot, cynefin, why5, firstprinciples, shuhari)",
                    true,
                ),
                ("problem", "string", "Problem description or context", true),
            ],
        ),
        make_tool(
            "cid_tanto_check",
            "Sanity-check a numeric value against known physical ranges. Categories: speed_mph, speed_ms, temp_c, height_m, weight_kg, energy_j, power_w, distance_km, time_s, price_usd, percent. Returns whether value is within expected range with typical values.",
            &[
                ("value", "number", "Numeric value to check", true),
                (
                    "category",
                    "string",
                    "Physical category (e.g. speed_mph, temp_c, height_m, weight_kg)",
                    true,
                ),
            ],
        ),
        make_tool(
            "cid_tanto_estimate",
            "Perform a Fermi order-of-magnitude estimate. Places a value in context with analogies (grain of sand, apple, human, car, planet, star, galaxy) and provides reference comparisons. Includes below/above reference values.",
            &[("value", "number", "Value to estimate", true)],
        ),
        make_tool(
            "cid_tanto_pipeline",
            "Evaluate a pipeline expression. Pipeline operator '|' passes results between segments. '_' refers to the previous segment's result. Example: 'div 1 3 | mul 6 _' computes (1/3)*6 = 2.",
            &[("expression", "string", "Pipeline expression", true)],
        ),
        make_tool(
            "cid_tanto_verify",
            "Verify an expression against an expected value. Returns OK (diff < 1e-10), CLOSE (diff < 0.01), or MISMATCH with computed and expected values. Example: verify 5 'hypot(3, 4)'",
            &[
                ("expected", "number", "Expected value", true),
                (
                    "expression",
                    "string",
                    "Expression to evaluate and compare",
                    true,
                ),
            ],
        ),
        make_tool(
            "cid_tanto_test",
            "Run Tanto's self-test suite. Tests all core math operations, constants, natural language processing, and pipeline evaluation. Returns pass/fail for each test with summary.",
            &[],
        ),
    ])
}

fn make_tool(name: &str, description: &str, params: &[(&str, &str, &str, bool)]) -> JsonValue {
    let mut properties = Vec::new();
    let mut required = Vec::new();

    for (pname, ptype, pdesc, preq) in params {
        properties.push((
            pname.to_string(),
            JsonValue::Object(vec![
                ("type".to_string(), JsonValue::Str(ptype.to_string())),
                ("description".to_string(), JsonValue::Str(pdesc.to_string())),
            ]),
        ));
        if *preq {
            required.push(JsonValue::Str(pname.to_string()));
        }
    }

    JsonValue::Object(vec![
        ("name".to_string(), JsonValue::Str(name.to_string())),
        (
            "description".to_string(),
            JsonValue::Str(description.to_string()),
        ),
        (
            "inputSchema".to_string(),
            JsonValue::Object(vec![
                ("type".to_string(), JsonValue::Str("object".to_string())),
                ("properties".to_string(), JsonValue::Object(properties)),
                ("required".to_string(), JsonValue::Array(required)),
            ]),
        ),
    ])
}

pub fn call_tool(
    name: &str,
    arguments: &JsonValue,
    engine: &mut InferenceEngine,
) -> Result<String, String> {
    let args = arguments.as_object().ok_or("arguments must be an object")?;

    match name {
        "cid_validate" => {
            let text = get_required_str(args, "text")?;
            let context = get_required_str(args, "context")?;
            let domain = get_optional_str(args, "domain");

            let mut request = ValidationRequest::new(text, context);
            if let Some(d) = domain {
                request = request.with_domain(d);
            }

            match engine.validate(request) {
                Ok(result) => {
                    let fix_count = result.fix_count();
                    let verdict = result.verdict().as_str().to_string();
                    Ok(stringify(&JsonValue::Object(vec![
                        (
                            "validated_text".to_string(),
                            JsonValue::Str(result.validated_text),
                        ),
                        (
                            "original_text".to_string(),
                            JsonValue::Str(result.original_text),
                        ),
                        (
                            "confidence".to_string(),
                            JsonValue::Number(result.confidence),
                        ),
                        ("verdict".to_string(), JsonValue::Str(verdict)),
                        ("fix_count".to_string(), JsonValue::Number(fix_count as f64)),
                        ("cost_usd".to_string(), JsonValue::Number(result.cost_usd)),
                    ])))
                }
                Err(e) => Err(format!("Validation error: {}", e)),
            }
        }
        "cid_fix" => {
            let text = get_required_str(args, "text")?;
            let context = get_required_str(args, "context")?;

            let (fixed, fixes) = engine.fix(text, context);

            let fix_list: Vec<JsonValue> = fixes
                .iter()
                .map(|f| {
                    JsonValue::Object(vec![
                        ("original".to_string(), JsonValue::Str(f.original.clone())),
                        ("fixed".to_string(), JsonValue::Str(f.fixed.clone())),
                        ("reason".to_string(), JsonValue::Str(f.reason.clone())),
                        ("confidence".to_string(), JsonValue::Number(f.confidence)),
                    ])
                })
                .collect();

            Ok(stringify(&JsonValue::Object(vec![
                ("original".to_string(), JsonValue::Str(text.to_string())),
                ("fixed".to_string(), JsonValue::Str(fixed)),
                ("fixes".to_string(), JsonValue::Array(fix_list)),
            ])))
        }
        "cid_lookup" => {
            let name = get_required_str(args, "name")?;
            match engine.lookup_fact(name) {
                Some(fact) => Ok(stringify(&JsonValue::Object(vec![
                    ("name".to_string(), JsonValue::Str(fact.name.clone())),
                    ("value".to_string(), JsonValue::Number(fact.value)),
                    ("unit".to_string(), JsonValue::Str(fact.unit.clone())),
                    ("source".to_string(), JsonValue::Str(fact.source.clone())),
                ]))),
                None => Err(format!("Fact '{}' not found", name)),
            }
        }
        "cid_search" => {
            let query = get_required_str(args, "query")?;
            let facts = engine.search_facts(query);

            let results: Vec<JsonValue> = facts
                .iter()
                .map(|f| {
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str(f.name.clone())),
                        ("value".to_string(), JsonValue::Number(f.value)),
                        ("unit".to_string(), JsonValue::Str(f.unit.clone())),
                        ("source".to_string(), JsonValue::Str(f.source.clone())),
                    ])
                })
                .collect();

            Ok(stringify(&JsonValue::Object(vec![
                ("query".to_string(), JsonValue::Str(query.to_string())),
                ("count".to_string(), JsonValue::Number(results.len() as f64)),
                ("results".to_string(), JsonValue::Array(results)),
            ])))
        }
        "cid_detect_fallacies" => {
            let text = get_required_str(args, "text")?;
            let findings = engine.detect_fallacies(text);

            let finding_list: Vec<JsonValue> = findings
                .iter()
                .map(|f| {
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str(f.name.to_string())),
                        (
                            "category".to_string(),
                            JsonValue::Str(f.category.to_string()),
                        ),
                        (
                            "description".to_string(),
                            JsonValue::Str(f.description.to_string()),
                        ),
                        (
                            "indicator".to_string(),
                            JsonValue::Str(f.indicator.to_string()),
                        ),
                        ("confidence".to_string(), JsonValue::Number(f.confidence)),
                    ])
                })
                .collect();

            let score = if findings.is_empty() {
                1.0
            } else {
                let total: f64 = findings.iter().map(|f| f.confidence).sum();
                (1.0 - total.min(1.0)).max(0.0)
            };

            Ok(stringify(&JsonValue::Object(vec![
                ("text".to_string(), JsonValue::Str(text.to_string())),
                (
                    "fallacy_count".to_string(),
                    JsonValue::Number(findings.len() as f64),
                ),
                ("credibility_score".to_string(), JsonValue::Number(score)),
                ("fallacies".to_string(), JsonValue::Array(finding_list)),
            ])))
        }
        "cid_sanity_check" => {
            let value = args
                .iter()
                .find(|(k, _)| k == "value")
                .and_then(|(_, v)| v.as_number())
                .ok_or("Missing required parameter: value")?;
            let category = get_required_str(args, "category")?;

            match engine.sanity_check(value, category) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("category".to_string(), JsonValue::Str(result.category)),
                    ("value".to_string(), JsonValue::Number(result.value)),
                    ("unit".to_string(), JsonValue::Str(result.unit)),
                    ("in_range".to_string(), JsonValue::Bool(result.in_range)),
                    ("in_typical".to_string(), JsonValue::Bool(result.in_typical)),
                    ("min".to_string(), JsonValue::Number(result.range.min)),
                    ("max".to_string(), JsonValue::Number(result.range.max)),
                    (
                        "typical_min".to_string(),
                        JsonValue::Number(result.range.typical_min),
                    ),
                    (
                        "typical_max".to_string(),
                        JsonValue::Number(result.range.typical_max),
                    ),
                    (
                        "description".to_string(),
                        JsonValue::Str(result.range.description.to_string()),
                    ),
                ]))),
                None => Err(format!("Unknown category: {}", category)),
            }
        }
        "cid_detect_biases" => {
            let text = get_required_str(args, "text")?;
            let findings = engine.detect_biases(text);

            let finding_list: Vec<JsonValue> = findings
                .iter()
                .map(|f| {
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str(f.name.to_string())),
                        (
                            "description".to_string(),
                            JsonValue::Str(f.description.to_string()),
                        ),
                        (
                            "mitigation".to_string(),
                            JsonValue::Str(f.mitigation.to_string()),
                        ),
                        (
                            "indicator".to_string(),
                            JsonValue::Str(f.indicator.to_string()),
                        ),
                        ("confidence".to_string(), JsonValue::Number(f.confidence)),
                    ])
                })
                .collect();

            let score = if findings.is_empty() {
                1.0
            } else {
                let total: f64 = findings.iter().map(|f| f.confidence).sum();
                (1.0 - total.min(1.0)).max(0.0)
            };

            Ok(stringify(&JsonValue::Object(vec![
                ("text".to_string(), JsonValue::Str(text.to_string())),
                (
                    "bias_count".to_string(),
                    JsonValue::Number(findings.len() as f64),
                ),
                ("objectivity_score".to_string(), JsonValue::Number(score)),
                ("biases".to_string(), JsonValue::Array(finding_list)),
            ])))
        }
        "cid_sample" => {
            let prompt = get_required_str(args, "prompt")?;
            let max_tokens = args
                .iter()
                .find(|(k, _)| k == "max_tokens")
                .and_then(|(_, v)| v.as_number())
                .unwrap_or(1000.0) as u32;

            let _ = max_tokens;

            Ok(stringify(&JsonValue::Object(vec![
                (
                    "status".to_string(),
                    JsonValue::Str("sampling_not_available".to_string()),
                ),
                (
                    "message".to_string(),
                    JsonValue::Str(
                        "MCP sampling requires client support. Use cid proxy for LLM completions."
                            .to_string(),
                    ),
                ),
                (
                    "prompt_preview".to_string(),
                    JsonValue::Str(prompt.chars().take(100).collect::<String>()),
                ),
            ])))
        }
        // === DYNAMIC KB TOOLS (Phase 1) ===
        "cid_kb_add" => {
            let name = get_required_str(args, "name")?;
            let value = args
                .iter()
                .find(|(k, _)| k == "value")
                .and_then(|(_, v)| v.as_number())
                .ok_or_else(|| "Missing required argument: value".to_string())?;
            let unit = get_required_str(args, "unit")?;
            let source = get_required_str(args, "source")?;
            let domain_name = args
                .iter()
                .find(|(k, _)| k == "domain")
                .and_then(|(_, v)| v.as_str())
                .unwrap_or("alpha");
            let domain = crate::kb::facts::Domain::from_name(domain_name)
                .unwrap_or(crate::kb::facts::Domain::Alpha);
            let confidence = args
                .iter()
                .find(|(k, _)| k == "confidence")
                .and_then(|(_, v)| v.as_number())
                .unwrap_or(0.8);
            let fact =
                crate::kb::facts::Fact::dynamic(name, value, unit, source, domain, confidence);
            engine.add_fact(fact.clone());
            Ok(stringify(&JsonValue::Object(vec![
                ("status".to_string(), JsonValue::Str("added".to_string())),
                ("name".to_string(), JsonValue::Str(name.to_string())),
                ("value".to_string(), JsonValue::Number(value)),
                ("unit".to_string(), JsonValue::Str(unit.to_string())),
                ("source".to_string(), JsonValue::Str(source.to_string())),
                (
                    "domain".to_string(),
                    JsonValue::Str(domain.description().to_string()),
                ),
                ("confidence".to_string(), JsonValue::Number(confidence)),
                (
                    "total_facts".to_string(),
                    JsonValue::Number(engine.kb_count() as f64),
                ),
            ])))
        }
        "cid_kb_context" => {
            let query = get_required_str(args, "query")?;
            let domain_name = get_required_str(args, "domain")?;
            let domain = crate::kb::facts::Domain::from_name(domain_name)
                .ok_or_else(|| format!("Unknown domain: '{}'. Use alpha..mu", domain_name))?;
            let max_results = args
                .iter()
                .find(|(k, _)| k == "max_results")
                .and_then(|(_, v)| v.as_number())
                .unwrap_or(10.0) as usize;
            // We need access to the KB directly via the pipeline
            // The engine doesn't expose context_window directly, so we use search_by_domain
            let results = engine.search_facts(query);
            let domain_filtered: Vec<&crate::kb::facts::Fact> = results
                .into_iter()
                .filter(|f| f.domain == domain)
                .take(max_results)
                .collect();
            let fact_list: Vec<JsonValue> = domain_filtered
                .iter()
                .map(|f| {
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str(f.name.clone())),
                        ("value".to_string(), JsonValue::Number(f.value)),
                        ("unit".to_string(), JsonValue::Str(f.unit.clone())),
                        ("source".to_string(), JsonValue::Str(f.source.clone())),
                        (
                            "domain".to_string(),
                            JsonValue::Str(f.domain.description().to_string()),
                        ),
                        ("confidence".to_string(), JsonValue::Number(f.confidence)),
                    ])
                })
                .collect();
            Ok(stringify(&JsonValue::Object(vec![
                ("query".to_string(), JsonValue::Str(query.to_string())),
                (
                    "domain".to_string(),
                    JsonValue::Str(domain.description().to_string()),
                ),
                (
                    "count".to_string(),
                    JsonValue::Number(fact_list.len() as f64),
                ),
                ("results".to_string(), JsonValue::Array(fact_list)),
            ])))
        }
        "cid_kb_recent" => {
            let within_secs = args
                .iter()
                .find(|(k, _)| k == "within_secs")
                .and_then(|(_, v)| v.as_number())
                .unwrap_or(3600.0) as u64;
            let all_facts = engine.kb_count();
            Ok(stringify(&JsonValue::Object(vec![
                ("total_facts".to_string(), JsonValue::Number(all_facts as f64)),
                ("note".to_string(), JsonValue::Str(
                    "Recent fact tracking available via context_window method. Use cid_kb_context for domain-filtered search.".to_string()
                )),
                ("within_secs".to_string(), JsonValue::Number(within_secs as f64)),
            ])))
        }
        // === TANTO MERGED TOOLS ===
        "cid_tanto_rational" => {
            let expr = get_required_str(args, "expression")?;
            let env = crate::tanto::TantoEnv::new();
            match crate::tanto::rational::eval_rational(expr, &env) {
                Some(rat) => Ok(stringify(&JsonValue::Object(vec![
                    ("expression".to_string(), JsonValue::Str(expr.to_string())),
                    ("rational".to_string(), JsonValue::Str(rat.format())),
                    ("mixed".to_string(), JsonValue::Str(rat.format_mixed())),
                    ("decimal".to_string(), JsonValue::Number(rat.to_f64())),
                ]))),
                None => {
                    // Fallback: show what f64 gives
                    match crate::tanto::evaluate_nl(expr, &env) {
                        Some(val) => Ok(stringify(&JsonValue::Object(vec![
                            ("expression".to_string(), JsonValue::Str(expr.to_string())),
                            (
                                "rational".to_string(),
                                JsonValue::Str("(not rational)".to_string()),
                            ),
                            ("decimal".to_string(), JsonValue::Number(val)),
                        ]))),
                        None => Err(format!("Cannot evaluate: '{}'", expr)),
                    }
                }
            }
        }
        "cid_tanto_eval" => {
            let expr = get_required_str(args, "expression")?;
            let env = crate::tanto::TantoEnv::new();
            match crate::tanto::evaluate_nl(expr, &env) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("expression".to_string(), JsonValue::Str(expr.to_string())),
                    ("result".to_string(), JsonValue::Number(result)),
                    (
                        "formatted".to_string(),
                        JsonValue::Str(crate::tanto::math::format_f64(result)),
                    ),
                ]))),
                None => Err(format!("Cannot evaluate: '{}'", expr)),
            }
        }
        "cid_tanto_convert" => {
            let value = args
                .iter()
                .find(|(k, _)| k == "value")
                .and_then(|(_, v)| v.as_number())
                .ok_or("Missing required parameter: value")?;
            let from = get_required_str(args, "from")?;
            let to = get_required_str(args, "to")?;

            let convert_args = format!("{} {} {}", value, from, to);
            match crate::tanto::convert::convert(&convert_args) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("value".to_string(), JsonValue::Number(value)),
                    ("from".to_string(), JsonValue::Str(from.to_string())),
                    ("to".to_string(), JsonValue::Str(to.to_string())),
                    ("result".to_string(), JsonValue::Number(result.value)),
                    (
                        "formatted".to_string(),
                        JsonValue::Str(format!(
                            "{} {} = {} {}",
                            value,
                            result.from,
                            crate::tanto::math::format_f64(result.value),
                            result.to
                        )),
                    ),
                ]))),
                None => Err(format!("Cannot convert: {} {} -> {}", value, from, to)),
            }
        }
        "cid_tanto_formula" => {
            let name = get_required_str(args, "name")?;
            let args_str = get_required_str(args, "args")?;
            let formula_args = format!("{} {}", name, args_str);

            match crate::tanto::formulas::compute_formula(&formula_args) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("name".to_string(), JsonValue::Str(result.name)),
                    ("formula".to_string(), JsonValue::Str(result.formula)),
                    (
                        "args".to_string(),
                        JsonValue::Array(
                            result.args.iter().map(|&a| JsonValue::Number(a)).collect(),
                        ),
                    ),
                    ("result".to_string(), JsonValue::Number(result.result)),
                    (
                        "formatted".to_string(),
                        JsonValue::Str(crate::tanto::math::format_f64(result.result)),
                    ),
                ]))),
                None => Err(format!(
                    "Unknown formula or bad args: {} {}",
                    name, args_str
                )),
            }
        }
        "cid_tanto_solve" => {
            let solver = get_required_str(args, "solver")?;
            let args_str = get_required_str(args, "args")?;
            let solve_args = format!("{} {}", solver, args_str);

            match crate::tanto::solver::solve(&solve_args) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("solver".to_string(), JsonValue::Str(result.solver)),
                    ("output".to_string(), JsonValue::Str(result.output)),
                ]))),
                None => Err(format!(
                    "Unknown solver or bad args: {} {}",
                    solver, args_str
                )),
            }
        }
        "cid_tanto_think" => {
            let framework = get_required_str(args, "framework")?;
            let problem = get_required_str(args, "problem")?;
            let think_args = format!("{} {}", framework, problem);

            match crate::tanto::thinking::think(&think_args) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("framework".to_string(), JsonValue::Str(result.framework)),
                    ("header".to_string(), JsonValue::Str(result.header)),
                    ("body".to_string(), JsonValue::Str(result.body)),
                ]))),
                None => Err(format!(
                    "Unknown framework: {}. Try: ooda, swot, cynefin, why5, firstprinciples, shuhari",
                    framework
                )),
            }
        }
        "cid_tanto_check" => {
            let value = args
                .iter()
                .find(|(k, _)| k == "value")
                .and_then(|(_, v)| v.as_number())
                .ok_or("Missing required parameter: value")?;
            let category = get_required_str(args, "category")?;
            let check_args = format!("{} as {}", value, category);

            match crate::tanto::sanity::check(&check_args) {
                Some(result) => {
                    let in_range = value >= result.min && value <= result.max;
                    let warning = if value < result.min {
                        Some("below expected range -- check units/formula")
                    } else if value > result.max {
                        Some("above expected range -- check units/formula")
                    } else {
                        None
                    };
                    Ok(stringify(&JsonValue::Object(vec![
                        ("category".to_string(), JsonValue::Str(result.category)),
                        ("value".to_string(), JsonValue::Number(result.val)),
                        ("unit".to_string(), JsonValue::Str(result.unit)),
                        ("min".to_string(), JsonValue::Number(result.min)),
                        ("max".to_string(), JsonValue::Number(result.max)),
                        ("typical".to_string(), JsonValue::Str(result.typical)),
                        ("in_range".to_string(), JsonValue::Bool(in_range)),
                        (
                            "warning".to_string(),
                            match warning {
                                Some(w) => JsonValue::Str(w.to_string()),
                                None => JsonValue::Null,
                            },
                        ),
                    ])))
                }
                None => Err(format!(
                    "Unknown category: {}. Available: speed_mph, speed_ms, temp_c, height_m, weight_kg, energy_j, power_w, distance_km, time_s, price_usd, percent",
                    category
                )),
            }
        }
        "cid_tanto_estimate" => {
            let value = args
                .iter()
                .find(|(k, _)| k == "value")
                .and_then(|(_, v)| v.as_number())
                .ok_or("Missing required parameter: value")?;

            match crate::tanto::sanity::estimate(&value.to_string()) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    ("value".to_string(), JsonValue::Number(result.val)),
                    (
                        "order_of_magnitude".to_string(),
                        JsonValue::Number(result.order as f64),
                    ),
                    ("context".to_string(), JsonValue::Str(result.context)),
                    (
                        "below_value".to_string(),
                        JsonValue::Number(result.below_val),
                    ),
                    ("below_name".to_string(), JsonValue::Str(result.below_name)),
                    (
                        "above_value".to_string(),
                        JsonValue::Number(result.above_val),
                    ),
                    ("above_name".to_string(), JsonValue::Str(result.above_name)),
                ]))),
                None => Err(format!("Cannot estimate: {}", value)),
            }
        }
        "cid_tanto_pipeline" => {
            let expression = get_required_str(args, "expression")?;
            let env = crate::tanto::TantoEnv::new();

            match crate::tanto::pipeline::evaluate_pipeline(expression, &env) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    (
                        "expression".to_string(),
                        JsonValue::Str(expression.to_string()),
                    ),
                    ("result".to_string(), JsonValue::Number(result)),
                    (
                        "formatted".to_string(),
                        JsonValue::Str(crate::tanto::math::format_f64(result)),
                    ),
                ]))),
                None => Err(format!("Cannot evaluate pipeline: '{}'", expression)),
            }
        }
        "cid_tanto_verify" => {
            let expected = args
                .iter()
                .find(|(k, _)| k == "expected")
                .and_then(|(_, v)| v.as_number())
                .ok_or("Missing required parameter: expected")?;
            let expression = get_required_str(args, "expression")?;
            let verify_args = format!("verify {} {}", expected, expression);
            let env = crate::tanto::TantoEnv::new();

            match crate::tanto::verify::verify(&verify_args, &env) {
                Some(result) => Ok(stringify(&JsonValue::Object(vec![
                    (
                        "expression".to_string(),
                        JsonValue::Str(expression.to_string()),
                    ),
                    ("expected".to_string(), JsonValue::Number(result.expected)),
                    ("computed".to_string(), JsonValue::Number(result.computed)),
                    ("diff".to_string(), JsonValue::Number(result.diff)),
                    (
                        "status".to_string(),
                        JsonValue::Str(result.status.to_string()),
                    ),
                ]))),
                None => Err(format!("Cannot verify: verify {} {}", expected, expression)),
            }
        }
        "cid_tanto_test" => {
            let results = crate::tanto::verify::run_self_test();
            let total = results.len();
            let passed = results.iter().filter(|(_, ok)| *ok).count();
            let failed = total - passed;

            let test_results: Vec<JsonValue> = results
                .iter()
                .map(|(name, ok)| {
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str(name.to_string())),
                        ("passed".to_string(), JsonValue::Bool(*ok)),
                    ])
                })
                .collect();

            Ok(stringify(&JsonValue::Object(vec![
                ("total".to_string(), JsonValue::Number(total as f64)),
                ("passed".to_string(), JsonValue::Number(passed as f64)),
                ("failed".to_string(), JsonValue::Number(failed as f64)),
                ("all_passed".to_string(), JsonValue::Bool(failed == 0)),
                ("results".to_string(), JsonValue::Array(test_results)),
            ])))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn get_required_str<'a>(args: &'a [(String, JsonValue)], key: &str) -> Result<&'a str, String> {
    args.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.as_str())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}

fn get_optional_str<'a>(args: &'a [(String, JsonValue)], key: &str) -> Option<&'a str> {
    args.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.as_str())
}
