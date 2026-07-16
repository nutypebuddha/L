use cid::core::ball::{Ball, TokenCandidate};
use cid::core::pin::{Gate, PinField};
use cid::core::pocket::Pocket;
use cid::economy::budget::Budget;
use cid::economy::tray::BallEconomy;
use cid::gates::GateValidator;
use cid::gates::{
    confidence::ConfidenceGate, fact::FactGate, formal::FormalGate, logic::LogicGate,
    math::MathGate,
};
use cid::inference::json::{stringify, JsonValue};
use cid::inference::{InferenceEngine, ProxyConfig, ProxyServer, ValidationRequest};
use cid::kb::facts::KnowledgeBase;
use cid::output::trace::TraceLog;
use cid::state::machine::StateMachine;
use std::io::{self, Read, Write};

struct CIDDevice {
    pin_field: PinField,
    state_machine: StateMachine,
    economy: BallEconomy,
    _budget: Budget,
    kb: KnowledgeBase,
    _trace_log: TraceLog,
}

impl CIDDevice {
    fn new() -> Self {
        CIDDevice {
            pin_field: PinField::new(),
            state_machine: StateMachine::new(),
            economy: BallEconomy::new(1000),
            _budget: Budget::new(1000000, 10.0),
            kb: KnowledgeBase::new(),
            _trace_log: TraceLog::new(100),
        }
    }

    fn validate_token(&mut self, token: &str, context: &str) -> Ball {
        let hash: u32 = token
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let logit = (hash % 1000) as f64 / 1000.0;
        let candidate = TokenCandidate::new(hash, token, logit);
        let mut ball = Ball::new(candidate);

        for pin in self.pin_field.pins.iter() {
            if !pin.enabled {
                continue;
            }

            let result = match pin.gate {
                Gate::Math => MathGate::new().validate(&mut ball, context),
                Gate::Logic => LogicGate::new().validate(&mut ball, context),
                Gate::Fact => FactGate::new(&self.kb).validate(&mut ball, context),
                Gate::Confidence => ConfidenceGate::new(pin.threshold).validate(&mut ball, context),
                Gate::Formal => FormalGate::new().validate(&mut ball, context),
            };

            ball.add_result(result);
        }

        ball
    }

    fn select_best(&self, candidates: Vec<Ball>) -> Option<Pocket> {
        Pocket::select_best(candidates)
    }

    fn show_state(&self) -> String {
        cid::output::format::format_state(self.state_machine.current()).to_string()
    }

    fn show_economy(&self) -> String {
        format!(
            "Economy:\n  Tray: {}\n  Spent: {}\n  Won: {}\n  Cost: ${:.6}\n  ROI: {:.1}%",
            self.economy.balance(),
            self.economy.total_spent(),
            self.economy.total_won(),
            self.economy.total_cost_usd(),
            self.economy.roi()
        )
    }

    fn show_pins(&self) -> String {
        let mut output = String::from("Pin Field:\n");
        for pin in &self.pin_field.pins {
            let status = if pin.enabled { "ENABLED" } else { "DISABLED" };
            output.push_str(&format!(
                "  {:?}: threshold={:.2}, cost={:.6}, status={}\n",
                pin.gate, pin.threshold, pin.cost, status
            ));
        }
        output
    }

    fn show_help(&self) -> String {
        r#"CID - Calibrated Inference Device
Per-token validation for LLMs using pachinko mechanics.

Commands:
  gate <token> <context>        Validate a token
  beam <tokens> <context>       Validate multiple tokens
  validate <text> <context>     Validate via inference API
  fix <text> --- <context>      Auto-fix mode (use --- to separate text from context)
  mcp                           Start MCP server (stdio JSON-RPC)
  proxy --port <port> --llm <url> --key <key>  Start proxy server
  state                         Show current state
  pins                          Show pin field
  pins adjust <gate> <thresh>   Adjust gate threshold
  economy                       Show ball economy
  compress <text> [level]       Compress prompt (light/medium/aggressive)
  score <text>                  Score response quality
   cache-stats                   Show semantic cache statistics
   kb list                       List knowledge base
   kb lookup <name>              Lookup fact
   tanto eval <expr>             Evaluate math expression (Tanto engine)
   tanto convert <val> <from> <to>  Unit conversion
   tanto formula <name> <args>   Compute physics/formula
   tanto solve <solver> <args>   Multi-step solver
   tanto think <fw> <problem>    Thinking framework
   tanto check <val> as <cat>    Sanity check
   tanto estimate <val>          Fermi estimate
    tanto pipeline <expr>         Pipeline evaluation
    tanto rational <expr>         Evaluate as exact rational (fraction)
    tanto verify <exp> <expr>     Verify expression
   tanto test                    Run Tanto self-test
   help                          Show this help
   test                          Run self-test
   version                       Show version
"#
        .to_string()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "proxy" => {
                let config = parse_proxy_args(&args[2..]);
                let server = ProxyServer::new(config);
                if let Err(e) = server.run() {
                    eprintln!("Proxy error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "mcp" => {
                cid::mcp::server::run();
                return;
            }
            "mcp-http" => {
                let addr = args.get(2).map(|s| s.as_str()).unwrap_or("127.0.0.1:8080");
                let server = cid::mcp::http::HttpServer::new(addr);
                if let Err(e) = server.run() {
                    eprintln!("HTTP MCP server error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            "validate" => {
                if args.len() < 4 {
                    eprintln!("Usage: cid validate <text> <context>");
                    std::process::exit(1);
                }
                let text = &args[2];
                let context = &args[3];
                let domain = args.get(4).map(|s| s.as_str());

                let mut engine = InferenceEngine::new();
                let mut request = ValidationRequest::new(text, context);
                if let Some(d) = domain {
                    request = request.with_domain(d);
                }

                match engine.validate(request) {
                    Ok(result) => {
                        let validated_text = result.validated_text.clone();
                        let confidence = result.confidence;
                        let passed = result.passed;
                        let fix_count = result.fix_count() as f64;

                        println!(
                            "{}",
                            stringify(&JsonValue::Object(vec![
                                ("validated_text".to_string(), JsonValue::Str(validated_text)),
                                ("confidence".to_string(), JsonValue::Number(confidence)),
                                ("passed".to_string(), JsonValue::Bool(passed)),
                                ("fix_count".to_string(), JsonValue::Number(fix_count)),
                            ]))
                        );
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            "fix" => {
                if args.len() < 4 {
                    eprintln!("Usage: cid fix <text> --- <context>");
                    std::process::exit(1);
                }
                let joined = args[2..].join(" ");
                let (text, context) = if let Some(pos) = joined.find(" --- ") {
                    (joined[..pos].to_string(), joined[pos + 5..].to_string())
                } else {
                    (args[2].clone(), args[3].clone())
                };

                let mut engine = InferenceEngine::new();
                let (fixed, fixes) = engine.fix(&text, &context);

                println!("Fixed: {}", fixed);
                if !fixes.is_empty() {
                    println!("\nFixes applied:");
                    for fix in &fixes {
                        println!("  {} -> {} ({})", fix.original, fix.fixed, fix.reason);
                    }
                }
                return;
            }
            "compress" => {
                if args.len() < 3 {
                    eprintln!("Usage: cid compress <text> [light|medium|aggressive]");
                    std::process::exit(1);
                }
                let text = &args[2];
                let level = match args.get(3).map(|s| s.as_str()) {
                    Some("light") => cid::inference::CompressionLevel::Light,
                    Some("aggressive") => cid::inference::CompressionLevel::Aggressive,
                    _ => cid::inference::CompressionLevel::Medium,
                };
                let compressor = cid::inference::PromptCompressor::new(level);
                let (compressed, stats) = compressor.compress(text);
                println!("Compressed: {}", compressed);
                println!("Original tokens: {}", stats.original_tokens);
                println!("Compressed tokens: {}", stats.compressed_tokens);
                println!(
                    "Saved: {} tokens ({:.1}%)",
                    stats.saved_tokens, stats.saved_percent
                );
                return;
            }
            "score" => {
                if args.len() < 3 {
                    eprintln!("Usage: cid score <text>");
                    std::process::exit(1);
                }
                let text = &args[2];
                let context = args.get(3).map(|s| s.as_str()).unwrap_or("general");
                let scorer = cid::inference::ResponseScorer::new();
                let report = scorer.score(text, context);
                println!("Quality Score: {:.2}", report.overall_score);
                println!("Confidence: {:.2}", report.confidence);
                println!("Action: {:?}", report.action);
                if !report.issues.is_empty() {
                    println!("Issues:");
                    for issue in &report.issues {
                        println!(
                            "  [{:?}] {}: {}",
                            issue.severity, issue.category, issue.description
                        );
                    }
                }
                return;
            }
            "--help" | "-h" | "help" => {
                // Standalone help (not REPL) — print and exit.
                let device = CIDDevice::new();
                print!("{}", device.show_help());
                return;
            }
            _ => {}
        }
    }

    let mut device = CIDDevice::new();
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap_or(0);
    let mut out = io::stdout();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let result = process_line(line, &mut device);
        let _ = out.write_all(result.as_bytes());
        let _ = out.write_all(b"\n");
    }
}

fn parse_proxy_args(args: &[String]) -> ProxyConfig {
    let mut port = "8080".to_string();
    let mut llm = String::new();
    let mut key = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--llm" => {
                if i + 1 < args.len() {
                    llm = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--key" if i + 1 < args.len() => {
                key = args[i + 1].clone();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    ProxyConfig::new(&format!("127.0.0.1:{}", port), &llm, &key)
}

fn process_line(line: &str, device: &mut CIDDevice) -> String {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    let cmd = parts[0];

    match cmd {
        "help" | "?" => device.show_help(),
        "version" | "v" => {
            "CID v0.1.0 -- Calibrated Inference Device + Tanto Compute Engine".to_string()
        }
        "state" => device.show_state(),
        "pins" => device.show_pins(),
        "economy" | "econ" => device.show_economy(),
        "gate" => {
            if parts.len() < 3 {
                return "Usage: gate <token> <context>".to_string();
            }
            let token = parts[1];
            let context = parts[2];
            let ball = device.validate_token(token, context);
            cid::output::format::format_ball(&ball)
        }
        "beam" => {
            if parts.len() < 3 {
                return "Usage: beam <token1,token2,...> <context>".to_string();
            }
            let tokens: Vec<&str> = parts[1].split(',').collect();
            let context = parts[2];
            let mut candidates = Vec::new();

            for token in tokens.iter() {
                let hash: u32 = token
                    .bytes()
                    .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
                let logit = (hash % 1000) as f64 / 1000.0;
                let candidate = TokenCandidate::new(hash, token, logit);
                let mut ball = Ball::new(candidate);

                for pin in device.pin_field.pins.iter() {
                    if !pin.enabled {
                        continue;
                    }
                    let result = match pin.gate {
                        Gate::Math => MathGate::new().validate(&mut ball, context),
                        Gate::Logic => LogicGate::new().validate(&mut ball, context),
                        Gate::Fact => FactGate::new(&device.kb).validate(&mut ball, context),
                        Gate::Confidence => {
                            ConfidenceGate::new(pin.threshold).validate(&mut ball, context)
                        }
                        Gate::Formal => FormalGate::new().validate(&mut ball, context),
                    };
                    ball.add_result(result);
                }
                candidates.push(ball);
            }

            let selected = device.select_best(candidates.clone());

            let mut output = String::new();
            output.push_str(&cid::output::format::format_validation_summary(
                &candidates,
                selected.as_ref(),
            ));
            output.push('\n');

            for ball in &candidates {
                output.push_str(&cid::output::format::format_ball(ball));
                output.push('\n');
            }

            if let Some(pocket) = selected {
                output.push_str(&cid::output::format::format_pocket(&pocket));
            } else {
                output.push_str("No valid token found.\n");
            }

            output
        }
        "validate" => {
            let (text, context) = if let Some(pos) = line.find(" --- ") {
                (&line[9..pos], &line[pos + 5..])
            } else if parts.len() >= 3 {
                let rest = &line[9..]; // after "validate "
                if let Some(last_space) = rest.rfind(' ') {
                    (&rest[..last_space], &rest[last_space + 1..])
                } else {
                    (rest, "")
                }
            } else if parts.len() == 2 {
                (&line[9..], "")
            } else {
                return "Usage: validate <text> <context>".to_string();
            };

            let mut engine = InferenceEngine::new();
            let request = ValidationRequest::new(text, context);

            match engine.validate(request) {
                Ok(result) => {
                    let mut output = String::new();
                    output.push_str(&format!("Validated: {}\n", result.validated_text));
                    output.push_str(&format!("Confidence: {:.4}\n", result.confidence));
                    output.push_str(&format!("Passed: {}\n", result.passed));
                    output.push_str(&format!("Fixes: {}\n", result.fix_count()));
                    output.push_str(&format!("State: {:?}\n", result.state));
                    output.push_str(&format!("Cost: ${:.6}\n", result.cost_usd));
                    output
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        "fix" => {
            let (text, context) = if let Some(pos) = line.find(" --- ") {
                (&line[4..pos], &line[pos + 5..])
            } else if parts.len() >= 3 {
                // No --- separator: everything after "fix " is text, last word is context
                let rest = &line[4..]; // after "fix "
                if let Some(last_space) = rest.rfind(' ') {
                    (&rest[..last_space], &rest[last_space + 1..])
                } else {
                    (rest, "")
                }
            } else if parts.len() == 2 {
                (&line[4..], "")
            } else {
                return "Usage: fix <text> --- <context>".to_string();
            };

            let mut engine = InferenceEngine::new();
            let (fixed, fixes) = engine.fix(text, context);

            let mut output = String::new();
            output.push_str(&format!("Original: {}\n", text));
            output.push_str(&format!("Fixed: {}\n", fixed));

            if !fixes.is_empty() {
                output.push_str("Fixes:\n");
                for fix in &fixes {
                    output.push_str(&format!(
                        "  {} -> {} ({})\n",
                        fix.original, fix.fixed, fix.reason
                    ));
                }
            }

            output
        }
        "kb" => {
            if parts.len() < 2 {
                return "Usage: kb list | kb lookup <name>".to_string();
            }
            match parts[1] {
                "list" => {
                    let mut output = String::from("Knowledge Base:\n");
                    for fact in device.kb.list_facts() {
                        output
                            .push_str(&format!("  {} = {} {}\n", fact.name, fact.value, fact.unit));
                    }
                    output
                }
                "lookup" => {
                    if parts.len() < 3 {
                        return "Usage: kb lookup <name>".to_string();
                    }
                    match device.kb.lookup(parts[2]) {
                        Some(fact) => format!(
                            "{} = {} {} [{}]",
                            fact.name, fact.value, fact.unit, fact.source
                        ),
                        None => format!("Fact '{}' not found", parts[2]),
                    }
                }
                _ => "Usage: kb list | kb lookup <name>".to_string(),
            }
        }
        "tanto" => {
            if parts.len() < 2 {
                return "Usage: tanto <subcommand> [args]. Try 'help' for full list.".to_string();
            }
            let subcmd = parts[1];
            let rest = if parts.len() > 2 {
                line[line.find(subcmd).unwrap_or(0) + subcmd.len() + 1..].trim()
            } else {
                ""
            };

            let env = &mut cid::tanto::TantoEnv::new();

            match subcmd {
                "eval" => match cid::tanto::evaluate_nl(rest, env) {
                    Some(val) => format!("= {}", cid::tanto::math::format_f64(val)),
                    None => format!("Error: cannot evaluate '{}'", rest),
                },
                "convert" => match cid::tanto::convert::convert(rest) {
                    Some(cr) => format!(
                        "= {} {} (from {} to {})",
                        cid::tanto::math::format_f64(cr.value),
                        cr.to,
                        cr.from,
                        cr.to
                    ),
                    None => format!("Error: unknown conversion '{}'", rest),
                },
                "formula" => match cid::tanto::formulas::compute_formula(rest) {
                    Some(fr) => format!(
                        "{} = {}  [{}]",
                        fr.name,
                        cid::tanto::math::format_f64(fr.result),
                        fr.formula
                    ),
                    None => format!("Error: unknown formula '{}'", rest),
                },
                "solve" => match cid::tanto::solver::solve(rest) {
                    Some(sr) => sr.output,
                    None => format!("Error: unknown solver '{}'", rest),
                },
                "think" => match cid::tanto::thinking::think(rest) {
                    Some(tr) => tr.body,
                    None => format!("Error: unknown framework '{}'", rest),
                },
                "check" => match cid::tanto::sanity::check(rest) {
                    Some(cr) => {
                        let status = if cr.val < cr.min {
                            "BELOW RANGE"
                        } else if cr.val > cr.max {
                            "ABOVE RANGE"
                        } else {
                            "OK"
                        };
                        format!(
                            "{} {} [range: {} - {}, typical: {}, status: {}]",
                            cid::tanto::math::format_f64(cr.val),
                            cr.unit,
                            cid::tanto::math::format_f64(cr.min),
                            cid::tanto::math::format_f64(cr.max),
                            cr.typical,
                            status
                        )
                    }
                    None => format!("Error: bad args '{}'", rest),
                },
                "estimate" => match cid::tanto::sanity::estimate(rest) {
                    Some(er) => format!(
                        "{} ~ 10^{} ({})",
                        cid::tanto::math::format_f64(er.val),
                        er.order,
                        er.context
                    ),
                    None => format!("Error: bad number '{}'", rest),
                },
                "pipeline" => match cid::tanto::pipeline::evaluate_pipeline(rest, env) {
                    Some(val) => format!("= {}", cid::tanto::math::format_f64(val)),
                    None => format!("Error: cannot evaluate pipeline '{}'", rest),
                },
                "rational" => match cid::tanto::rational::eval_rational(rest, env) {
                    Some(rat) => format!(
                        "= {}  (decimal: {}, mixed: {})",
                        rat.format(),
                        rat.to_f64(),
                        rat.format_mixed()
                    ),
                    None => format!("Error: cannot evaluate '{}' as rational", rest),
                },
                "verify" => match cid::tanto::verify::verify(rest, env) {
                    Some(vr) => format!(
                        "{} (expected={}, computed={}, diff={})",
                        vr.status,
                        cid::tanto::math::format_f64(vr.expected),
                        cid::tanto::math::format_f64(vr.computed),
                        cid::tanto::math::format_f64(vr.diff)
                    ),
                    None => "Error: verify needs: verify <expected> <expr>".to_string(),
                },
                "test" => {
                    // Self-test (correctness checks)
                    let correctness = cid::tanto::verify::run_self_test();
                    let mut out = String::from("=== TANTO CORRECTNESS ===\n");
                    let mut pass = 0;
                    for (desc, ok) in &correctness {
                        pass += if *ok { 1 } else { 0 };
                        out.push_str(&format!("  {} {}\n", if *ok { "OK" } else { "FAIL" }, desc));
                    }
                    out.push_str(&format!(
                        "--- {} passed, {} failed ---\n\n",
                        pass,
                        correctness.len() - pass
                    ));

                    // Determinism audit
                    out.push_str("=== TANTO DETERMINISM ===\n");
                    let determinism = cid::tanto::determinism::run_determinism_audit();
                    let mut det_pass = 0;
                    for (desc, ok) in &determinism {
                        det_pass += if *ok { 1 } else { 0 };
                        out.push_str(&format!("  {} {}\n", if *ok { "OK" } else { "FAIL" }, desc));
                    }
                    out.push_str(&format!(
                        "--- {} passed, {} failed ---\n\n",
                        det_pass,
                        determinism.len() - det_pass
                    ));

                    out.push_str(&format!(
                        "=== SUMMARY: {} correctness + {} determinism tests ===\n",
                        pass, det_pass
                    ));
                    out
                }
                _ => format!("Unknown tanto subcommand: {}. Try 'help'.", subcmd),
            }
        }
        "test" => {
            let mut output = String::from("=== CID SELF-TEST ===\n");

            let ball = device.validate_token("42", "number");
            output.push_str(&format!(
                "Math test: {}\n",
                if ball.all_passed() { "PASS" } else { "FAIL" }
            ));

            let ball = device.validate_token("therefore", "because all men are mortal");
            output.push_str(&format!(
                "Logic test: {}\n",
                if ball.all_passed() { "PASS" } else { "FAIL" }
            ));

            let ball = device.validate_token("3.14159", "pi is approximately");
            output.push_str(&format!(
                "Fact test: {}\n",
                if ball.all_passed() { "PASS" } else { "FAIL" }
            ));

            output.push_str("=== TESTS COMPLETE ===\n");
            output
        }
        _ => format!("Unknown command: {}. Try 'help'.", cmd),
    }
}
