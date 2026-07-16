//! # Athena CLI
//!
//! Athena's command-line interface. Operates across the 4-layer security architecture:
//!
//! ```text
//! Asauchi  →  Zanpakuto  →  Shikai  →  Bankai
//! (public)    (access)     (queries)   (solved)
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Layer 1 — Asauchi (public interface)
//! athena info
//! athena validate "2 + 2" --gate math
//!
//! # Layer 2 — Zanpakuto (access control)
//! athena register --name user1 --tier shikai
//! athena login --session session_user1
//!
//! # Layer 3 — Shikai (query processing)
//! athena shikai "calculate force mass=5 acceleration=9.8"
//! athena shikai "traverse from surya"
//!
//! # Layer 4 — Bankai (atomic solve)
//! athena solve "force mass=5 acceleration=9.8"
//! athena chain --formulas "newtons_second,pythagorean" --args "mass=5,acceleration=9.8,a=3,b=4"
//! athena compose --formulas "newtons_second,momentum"
//!
//! # Cross-layer
//!
//! Copyright (c) 2026 nutypebuddha. All rights reserved.
//! Licensed under Athena Proprietary License. See LICENSE for terms.
//! Unauthorized copying, distribution, or reverse engineering is strictly prohibited.
//! athena wheel
//! athena search "momentum"
//! athena pipeline "calculate force" --identity admin
//! ```

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::collections::HashMap;

use clap::{Parser, Subcommand};

use athena::asauchi::Asauchi;
use athena::astrology::Aspect;
use athena::bankai::Bankai;
#[cfg(feature = "budget")]
use athena::budget::TokenBudget;
use athena::descent::DescentEngine;
use athena::entity::EntityRegistry;
use athena::formula::FormulaRegistry;
use athena::gates::GateResult;
use athena::gyro::GyroState;
use athena::shikai::Shikai;
use athena::wheel::{Domain, WheelGraph};
use athena::zanpakuto::nlp::NlpEngine;
use athena::zanpakuto::{AccessTier, Zanpakuto};

/// Athena — relational intelligence engine.
#[derive(Parser)]
#[command(
    name = "athena",
    version,
    about = "Relational intelligence — formulas, not facts"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ─── Layer 1: Asauchi ──────────────────────────────────
    /// Show system information (Asauchi layer)
    Info,

    /// Validate an expression or claim (Asauchi layer)
    Validate {
        /// Text to validate
        text: String,
        /// Gate to use
        #[arg(long, default_value = "math")]
        gate: String,
    },

    // ─── Layer 2: Zanpakuto ────────────────────────────────
    /// Register a new identity (Zanpakuto layer)
    Register {
        /// Identity name
        #[arg(long)]
        name: String,
        /// Access tier (asauchi, shikai, bankai)
        #[arg(long, default_value = "shikai")]
        tier: String,
    },

    /// List registered identities
    Identities,

    // ─── Layer 3: Shikai ───────────────────────────────────
    /// Process a query through Shikai (show intent/parsing)
    Shikai {
        /// Query string
        query: String,
    },

    // ─── Layer 4: Bankai ───────────────────────────────────
    /// Solve a query atomically (Bankai layer)
    Solve {
        /// Query to solve
        query: String,
        /// Identity to use
        #[arg(long, default_value = "default")]
        identity: String,
    },

    /// Evaluate a formula with arguments
    Eval {
        /// Formula ID
        #[arg(long)]
        formula: String,
        /// Arguments as key=value pairs
        #[arg(long, value_parser = parse_key_val)]
        args: Vec<KeyVal>,
    },

    /// Execute a chain of formulas
    Chain {
        /// Comma-separated formula IDs
        #[arg(long)]
        formulas: String,
        /// Arguments as key=value pairs
        #[arg(long, value_parser = parse_key_val)]
        args: Vec<KeyVal>,
    },

    /// Compose formulas into a reasoning chain
    Compose {
        /// Comma-separated formula IDs
        #[arg(long)]
        formulas: String,
    },

    /// Traverse the wheel graph from a domain
    Traverse {
        /// Starting domain
        #[arg(long)]
        domain: String,
        /// Maximum depth
        #[arg(long, default_value_t = 5)]
        depth: usize,
    },

    // ─── Cross-layer ──────────────────────────────────────
    /// Display the zodiac wheel
    Wheel {
        /// Domain to show (optional)
        #[arg(long)]
        domain: Option<String>,
    },

    /// Search the formula database
    Search {
        /// Search keyword
        keyword: String,
    },

    /// Find the shortest formula chain to derive a desired output
    Reason {
        /// Comma-separated variable names you already have values for
        #[arg(long)]
        have: String,
        /// The variable name you want to derive
        #[arg(long)]
        want: String,
        /// Maximum search depth
        #[arg(long, default_value_t = 5)]
        max_depth: usize,
        /// Execute the chain with the given args (instead of just showing the path)
        #[arg(long)]
        execute: bool,
        /// Arguments as key=value pairs (used with --execute)
        #[arg(long, value_parser = parse_key_val)]
        args: Vec<KeyVal>,
        /// Output without astrological names (use domain knowledge labels instead)
        #[arg(long)]
        neutral: bool,
    },

    /// Find a formula chain grounded in an entity's properties
    EntityReason {
        /// Entity ID to ground the reasoning in
        #[arg(long)]
        entity: String,
        /// The variable name you want to derive
        #[arg(long)]
        want: String,
        /// Maximum search depth
        #[arg(long, default_value_t = 5)]
        max_depth: usize,
        /// Execute the chain with entity properties as args
        #[arg(long)]
        execute: bool,
        /// Output without astrological names (use domain knowledge labels instead)
        #[arg(long)]
        neutral: bool,
        /// Additional arguments as key=value pairs (supplement entity properties)
        #[arg(long, value_parser = parse_key_val)]
        args: Vec<KeyVal>,
    },

    /// Show the full pipeline for a query
    Pipeline {
        /// Query to trace through the pipeline
        query: String,
        /// Identity name
        #[arg(long, default_value = "default")]
        identity: String,
    },

    /// Start the MCP stdio server
    Mcp,

    // ─── Entity layer ──────────────────────────────────────
    /// List all entities
    EntityList,
    /// Get an entity by ID
    EntityGet {
        /// Entity ID to retrieve
        id: String,
    },
    /// Compute aspect between two entities
    EntityAspect {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    /// Search entities by keyword
    EntitySearch { keyword: String },
    /// Evaluate a formula grounded in an entity
    EntityEval {
        /// Formula ID
        #[arg(long)]
        formula: String,
        /// Entity ID to ground in
        #[arg(long)]
        entity: String,
        /// Additional arguments as key=value pairs
        #[arg(long, value_parser = parse_key_val)]
        args: Vec<KeyVal>,
    },

    // ─── Classification layer ──────────────────────────────
    /// Classify a token through the Change Sorter (7-axis astrology)
    Classify {
        /// Token to classify
        token: String,
    },

    /// Show the current state of the gyroscopic wheel
    Gyro,

    // ─── Descent layer ─────────────────────────────────────
    /// Analyze a query through the 7-layer descent engine
    Descent {
        /// Query to analyze
        query: String,
    },

    /// Show the settling matrix for a query
    DescentMatrix {
        /// Query to analyze
        query: String,
    },

    // ─── Ephemeris layer ───────────────────────────────────
    /// Compute deterministic graha positions for a date (VSOP87/ELP-2000)
    Ephemeris {
        /// Date as YYYY-MM-DD (Gregorian, UT)
        #[arg(long)]
        date: String,
        /// Time of day as HH:MM (UT), defaults to 00:00
        #[arg(long, default_value = "00:00")]
        time: String,
    },

    // ─── LLM Router (Capstone) ───────────────────────────
    /// Distill the registry into (query → intent) training pairs (JSONL)
    /// for fine-tuning the capstone — the KB generates its own curriculum
    #[cfg(feature = "llm")]
    Distill {
        /// Output JSONL file (default: stdout)
        #[arg(long)]
        out: Option<String>,
        /// Cap the number of pairs
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Query the embedded LLM copilot (Qwen2.5-0.5B) for routing or generation
    #[cfg(feature = "llm")]
    Llm {
        /// Query to route through the LLM capstone
        query: String,
        /// Show full routing decision (not just the tool call)
        #[arg(long)]
        verbose: bool,
        /// Backend type: "local" or "remote"
        #[arg(long)]
        backend: Option<String>,
        /// Path to GGUF model file (local backend only)
        #[arg(long)]
        model: Option<String>,
        /// Endpoint URL (remote backend only)
        #[arg(long)]
        endpoint: Option<String>,
        /// Show backend health instead of routing
        #[arg(long)]
        health: bool,
        /// Free-form generation (don't try to parse as routing JSON)
        #[arg(long)]
        generate: bool,
    },

    // ─── Budget layer ──────────────────────────────────────
    /// Show token usage (every token is an entity)
    #[cfg(feature = "budget")]
    Budget,
    /// Reset the token budget
    #[cfg(feature = "budget")]
    BudgetReset,
}

#[derive(Debug, Clone)]
struct KeyVal {
    key: String,
    value: f64,
}

fn parse_key_val(s: &str) -> Result<KeyVal, String> {
    let s = s.trim();
    // Reject JSON-like input with a helpful message
    if s.starts_with('{') || s.starts_with('[') {
        return Err(format!(
            "unsupported format: '{s}'. Expected 'key=value' (e.g. --args mass=5.0). \
             Use multiple --args for multiple values: --args mass=5 --args acceleration=9.8"
        ));
    }
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --args format: '{s}'. Expected 'key=value' (e.g. --args mass=5.0). \
             Use multiple --args for multiple values: --args mass=5 --args acceleration=9.8"
        ));
    }
    let key = parts[0].trim().to_string();
    if key.is_empty() {
        return Err(format!(
            "empty key in --args '{s}'. Expected 'key=value' (e.g. --args mass=5.0)"
        ));
    }
    let value = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("invalid number '{}': {}", parts[1].trim(), e))?;
    Ok(KeyVal { key, value })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Core components
    let registry = build_default_registry();
    #[allow(unused_mut)]
    let mut entities = build_entity_registry();
    #[cfg(feature = "budget")]
    let mut budget = TokenBudget::new(2000, 1000, 3000);
    let asauchi = Asauchi::new();
    let mut zanpakuto = Zanpakuto::new();
    let shikai = Shikai::with_entities(registry.clone(), entities.clone());
    let mut bankai = Bankai::with_entities(registry.clone(), entities.clone());
    let nlp_engine = NlpEngine::new(entities.clone(), registry.clone());
    let wheel = WheelGraph::new();

    // Descent engine
    let descent_engine = DescentEngine::new(registry.clone(), entities.clone());

    // Gyro state — calibrated by settling matrix from descent engine
    let mut gyro_state = GyroState::new();

    // Pre-register a default identity
    let default_identity = zanpakuto.register("default", AccessTier::Bankai);

    match cli.command {
        // ─── Asauchi layer ────────────────────────────────
        Commands::Info => {
            let info = asauchi.info();
            println!("{}", serde_json::to_string_pretty(&info).unwrap());
        }

        Commands::Validate { text, gate } => {
            let result = asauchi.public_validate(&text, &gate);
            print_validation(&result);
        }

        // ─── Zanpakuto layer ──────────────────────────────
        Commands::Register { name, tier } => {
            let access_tier = match tier.to_lowercase().as_str() {
                "bankai" => AccessTier::Bankai,
                "shikai" => AccessTier::Shikai,
                "zanpakuto" => AccessTier::Zanpakuto,
                "asauchi" => AccessTier::Asauchi,
                _ => {
                    eprintln!("Warning: unknown tier '{tier}'. Defaulting to Asauchi. Valid tiers: asauchi, zanpakuto, shikai, bankai");
                    AccessTier::Asauchi
                }
            };
            let identity = zanpakuto.register(&name, access_tier);
            println!("Registered: {} ({:?})", identity.name, identity.tier);
            println!("Session: {}", identity.session);
            println!(
                "Capabilities: {:?}",
                identity
                    .capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
            );
        }

        Commands::Identities => {
            println!("=== Zanpakuto: Registered Identities ===");
            println!(
                "Default: {} ({:?})",
                default_identity.name, default_identity.tier
            );
        }

        // ─── Shikai layer ─────────────────────────────────
        Commands::Shikai { query } => {
            let nlp_ctx = nlp_engine.preprocess(&query);

            // Full pipeline: Descent → Gyro → Shikai
            let matrix = descent_engine.resolve_nlp(&nlp_ctx);
            gyro_state.apply_matrix(&matrix);
            gyro_state.update(0.1);

            match shikai.process_with_context(
                &query,
                &default_identity,
                Some(&nlp_ctx),
                Some(&matrix),
                Some(&gyro_state),
            ) {
                Ok(shikai_query) => {
                    println!("=== Shikai: Query Processed ===");
                    println!("{}", athena::shikai::Shikai::format_query(&shikai_query));
                    println!("\n=== NLP Context ===");
                    println!(
                        "  Intent:    {:?} (score: {:.2})",
                        nlp_ctx.likely_intent,
                        nlp_ctx.intent_scores.first().map_or(0.0, |(_, s)| *s)
                    );
                    if let Some(d) = nlp_ctx.likely_domain {
                        println!("  Domain:    {} ({})", d.symbol(), d.full_name());
                    }
                    if let Some(ref e) = nlp_ctx.likely_entity {
                        println!("  Entity:    {}", e);
                    }
                    println!("  QueryType: {:?}", nlp_ctx.query_type);
                    println!(
                        "  Tokens:    {} ({} significant)",
                        nlp_ctx.tokens.len(),
                        nlp_ctx.significant_tokens.len()
                    );
                    println!("\n=== Descent Matrix ===");
                    println!("{}", matrix.format());
                    println!("\n=== Gyro State ===");
                    let (sign, strength) = gyro_state.dominant_sign_info();
                    println!("  Dominant: {:?} (strength: {:.3})", sign, strength);
                    println!("  Current sign: {:?}", gyro_state.current_sign());
                    println!("  Precession: {:.3}", gyro_state.precession());
                }
                Err(e) => {
                    eprintln!("Shikai error: {e}");
                }
            }
        }

        // ─── Bankai layer ─────────────────────────────────
        Commands::Solve { query, identity } => {
            let id = zanpakuto
                .authenticate(&format!("session_{}", identity))
                .unwrap_or(&default_identity);
            let nlp_ctx = nlp_engine.preprocess(&query);

            // Full pipeline: Descent → Gyro → Shikai → Bankai
            let matrix = descent_engine.resolve_nlp(&nlp_ctx);
            gyro_state.apply_matrix(&matrix);
            gyro_state.update(0.1);
            bankai.gyro.gyro_state_mut().apply_matrix(&matrix);
            bankai.gyro.gyro_state_mut().update(0.1);

            match shikai.process_with_context(
                &query,
                id,
                Some(&nlp_ctx),
                Some(&matrix),
                Some(&gyro_state),
            ) {
                Ok(shikai_query) => {
                    let solve = bankai.solve(&shikai_query, id);
                    print_solve(&solve);
                }
                Err(e) => {
                    // Shikai couldn't parse intent — try direct formula evaluation
                    match bankai.evaluate(&query, &HashMap::new()) {
                        Ok(value) => {
                            println!("Direct eval: {query} = {value:.6}");
                        }
                        Err(_) => {
                            eprintln!("Bankai error: {e}");
                        }
                    }
                }
            }
        }

        Commands::Eval { formula, args } => {
            let mut args_map = HashMap::new();
            for kv in &args {
                args_map.insert(kv.key.clone(), kv.value);
            }
            // kind=llm formulas evaluate through the capstone; attach it
            // lazily so plain math evals never pay model-load time.
            #[cfg(feature = "llm")]
            let bankai = match registry.get(&formula).map(|f| f.formula_type) {
                Some(athena::formula::FormulaType::Llm) => {
                    use athena::inference::config::InferenceConfig;
                    use athena::llm::LlmRouter;
                    match LlmRouter::from_config(InferenceConfig::load(Default::default())) {
                        Ok(router) => bankai.with_capstone(router),
                        Err(e) => {
                            eprintln!("capstone unavailable ({e}); evaluation will fail");
                            bankai
                        }
                    }
                }
                _ => bankai,
            };
            match bankai.evaluate(&formula, &args_map) {
                Ok(value) => {
                    println!("Bankai: {formula} → {value:.6}");
                }
                Err(e) => eprintln!("Evaluation failed: {e}"),
            }
        }

        Commands::Chain { formulas, args } => {
            let formula_ids: Vec<&str> = formulas.split(',').map(|s| s.trim()).collect();
            let mut args_map = HashMap::new();
            for kv in &args {
                args_map.insert(kv.key.clone(), kv.value);
            }
            match bankai.chain(&formula_ids, &args_map) {
                Ok(result) => println!("{}", result.format()),
                Err(e) => eprintln!("Chain failed: {e}"),
            }
        }

        Commands::Compose { formulas } => {
            let formula_ids: Vec<&str> = formulas.split(',').map(|s| s.trim()).collect();
            match bankai.compose(&formula_ids) {
                Ok(comp) => {
                    println!("Composition:");
                    println!("  {}", comp.description);
                }
                Err(e) => eprintln!("Composition failed: {e}"),
            }
        }

        Commands::Traverse { domain, depth } => {
            let start = match Domain::parse(&domain) {
                Some(d) => d,
                None => {
                    eprintln!("Error: unknown domain '{domain}'.");
                    eprintln!("Valid domains: aries, taurus, gemini, cancer, leo, virgo, libra, scorpio, sagittarius, capricorn, aquarius, pisces");
                    return Ok(());
                }
            };
            let traversal = bankai.traverse(start, depth);
            println!("=== Bankai Traversal ===");
            println!("Path: {}", traversal.format_path());
            println!("Domains visited: {}", traversal.domains_visited().len());
            println!("Formulas found: {}", traversal.formula_count());
            for (i, step) in traversal.path.iter().enumerate() {
                println!(
                    "  {}. {}{} — {} formula(s): {}",
                    i + 1,
                    step.domain.symbol(),
                    step.domain.full_name(),
                    step.formulas_at_node.len(),
                    step.formulas_at_node.join(", "),
                );
            }
        }

        // ─── Cross-layer ──────────────────────────────────
        Commands::Wheel { domain } => {
            if let Some(d) = domain {
                if let Some(parsed) = Domain::parse(&d) {
                    let node = wheel.node(parsed);
                    println!("{} {} — {}", node.symbol, node.name, node.description);
                    println!("  Knowledge: {}", parsed.knowledge_domain());
                    println!(
                        "  Opposite: {} {}",
                        node.opposite.symbol(),
                        node.opposite.full_name()
                    );
                    println!("  Trines:");
                    for t in parsed.trines() {
                        println!("    {} {}", t.symbol(), t.full_name());
                    }
                    println!("  Adjacent:");
                    for a in parsed.adjacent() {
                        println!("    {} {}", a.symbol(), a.full_name());
                    }
                } else {
                    eprintln!("Unknown domain: {d}");
                }
            } else {
                println!("{}", wheel.render_wheel());
                println!("\nDomains:");
                for node in wheel.all_nodes() {
                    println!(
                        "  {} {:12} {:30} ↕ {}",
                        node.symbol,
                        node.name,
                        node.description,
                        node.opposite.full_name()
                    );
                }
            }
        }

        Commands::Search { keyword } => {
            let results = registry.search(&keyword);
            if results.is_empty() {
                println!("No formulas found for '{keyword}'.");
                println!("Athena stores formulas, not facts. Try a broader concept.");
            } else {
                println!("Found {} formula(s) for '{keyword}':", results.len());
                for f in results {
                    let type_str = match f.formula_type {
                        athena::formula::FormulaType::Math => "math",
                        athena::formula::FormulaType::Logic => "logic",
                        athena::formula::FormulaType::Llm => "llm",
                    };
                    println!(
                        "  {:<24} | {:12} | {:6} | {}",
                        f.id,
                        format!("{}{}", f.domain.symbol(), f.domain.full_name()),
                        type_str,
                        f.description,
                    );
                }
            }
        }

        Commands::Reason {
            have,
            want,
            max_depth,
            execute,
            args,
            neutral,
        } => {
            let have_vars: Vec<String> = have.split(',').map(|s| s.trim().to_string()).collect();
            match bankai.find_path(&have_vars, &want, max_depth) {
                Ok(path) => {
                    println!("=== Reason: {} → {} ===", have, want);
                    if path.is_empty() {
                        println!("Already have '{}' — no derivation needed.", want);
                        return Ok(());
                    }
                    println!("Found path ({} step(s)): {}", path.len(), path.join(" → "));
                    println!("\nFormula chain:");
                    for (i, fid) in path.iter().enumerate() {
                        if let Some(f) = registry.get(fid) {
                            let satisfied: Vec<&str> = f.inputs.iter().map(|_| "✓").collect();
                            let domain_label = if neutral {
                                f.domain.knowledge_domain()
                            } else {
                                f.domain.symbol()
                            };
                            println!(
                                "  {}. {} [{}]: {} → {}  inputs: {}",
                                i + 1,
                                fid,
                                domain_label,
                                f.inputs.join(", "),
                                f.output,
                                satisfied.join(", ")
                            );
                        }
                    }

                    // Check for variable-name collisions across domains
                    let mut collisions: Vec<String> = Vec::new();
                    for (i, fid) in path.iter().enumerate() {
                        if let Some(f) = registry.get(fid) {
                            for input in &f.inputs {
                                for (j, other_fid) in path.iter().enumerate() {
                                    if i == j {
                                        continue;
                                    }
                                    if let Some(other) = registry.get(other_fid) {
                                        if other.output == *input && other.domain != f.domain {
                                            let domain_a = if neutral {
                                                f.domain.knowledge_domain()
                                            } else {
                                                f.domain.full_name()
                                            };
                                            let domain_b = if neutral {
                                                other.domain.knowledge_domain()
                                            } else {
                                                other.domain.full_name()
                                            };
                                            collisions.push(format!(
                                                "  ⚠  Step {} input '{}' from {} is produced by Step {} in {} (same name, different domain — verify semantic match)",
                                                i + 1, input, domain_a, j + 1, domain_b,
                                            ));
                                        }
                                    }
                                }
                                for p in registry.all() {
                                    if p.output == *input
                                        && p.domain != f.domain
                                        && !path.contains(&p.id)
                                    {
                                        let domain_a = if neutral {
                                            f.domain.knowledge_domain()
                                        } else {
                                            f.domain.full_name()
                                        };
                                        collisions.push(format!(
                                            "  ⚠  Step {} input '{}' is also produced by '{}' in {} (different domain — verify this is the right match)",
                                            i + 1, input, p.id, domain_a,
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    collisions.sort();
                    collisions.dedup();
                    if !collisions.is_empty() {
                        println!("\n⚠  Potential variable-name collisions:");
                        for c in &collisions {
                            println!("{}", c);
                        }
                        println!("  Re-run with --execute to verify the chain actually works.");
                    }

                    if execute {
                        println!("\nExecuting chain...");
                        let mut args_map = HashMap::new();
                        for kv in &args {
                            args_map.insert(kv.key.clone(), kv.value);
                        }
                        let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                        match bankai.chain(&refs, &args_map) {
                            Ok(result) => {
                                if neutral {
                                    // Strip domain symbols from chain output
                                    let fmt = result.format();
                                    let neutral_fmt: String = fmt
                                        .lines()
                                        .map(|line| {
                                            let mut l = line.to_string();
                                            for d in athena::wheel::ALL_DOMAINS.iter() {
                                                l = l.replace(d.symbol(), "");
                                                l = l.replace(d.full_name(), d.knowledge_domain());
                                            }
                                            l
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    println!("\n{}", neutral_fmt);
                                } else {
                                    println!("\n{}", result.format());
                                }
                            }
                            Err(e) => {
                                eprintln!("Chain execution failed: {e}");
                            }
                        }
                    } else {
                        println!(
                            "\nTip: re-run with --execute and --args key=value to run the chain."
                        );
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!(
                        "\nNo chain found to derive '{}' from '{:?}'.",
                        want, have_vars
                    );
                    eprintln!("Try increasing --max-depth or checking the formula registry.");
                }
            }
        }

        Commands::EntityReason {
            entity,
            want,
            max_depth,
            execute,
            neutral,
            args,
        } => {
            let seed = match entities.get_seed(&entity) {
                Some(s) => s,
                None => {
                    eprintln!("Entity not found: '{}'", entity);
                    eprintln!("Use `athena entity-search <keyword>` to find entities.");
                    return Ok(());
                }
            };

            // Collect entity property names as available variables
            let mut have_vars: Vec<String> = seed.properties.keys().cloned().collect();
            // Also add entity formula output if it exists
            if let Some(ref formula) = seed.formula {
                // Try to derive the formula's output from the entity
                if let Some(f) = registry.get(formula) {
                    if !have_vars.contains(&f.output) {
                        have_vars.push(f.output.clone());
                    }
                }
            }

            if have_vars.is_empty() {
                eprintln!("Entity '{}' has no properties to reason from.", entity);
                return Ok(());
            }

            println!("=== Entity Reason: {} → {} ===", entity, want);
            println!(
                "Entity properties ({}): {}",
                have_vars.len(),
                have_vars.join(", ")
            );

            match bankai.find_path(&have_vars, &want, max_depth) {
                Ok(path) => {
                    if path.is_empty() {
                        println!(
                            "Already have '{}' in entity properties — no derivation needed.",
                            want
                        );
                        return Ok(());
                    }
                    println!("Found path ({} step(s)): {}", path.len(), path.join(" → "));
                    println!("\nFormula chain:");
                    for (i, fid) in path.iter().enumerate() {
                        if let Some(f) = registry.get(fid) {
                            let satisfied: Vec<&str> = f.inputs.iter().map(|_| "✓").collect();
                            let domain_label = if neutral {
                                f.domain.knowledge_domain()
                            } else {
                                f.domain.symbol()
                            };
                            println!(
                                "  {}. {} [{}]: {} → {}  inputs: {}",
                                i + 1,
                                fid,
                                domain_label,
                                f.inputs.join(", "),
                                f.output,
                                satisfied.join(", ")
                            );
                        }
                    }

                    if !execute {
                        println!(
                            "\nExecutable: re-run with --execute to evaluate using entity '{}'s properties.",
                            entity
                        );
                        return Ok(());
                    }

                    println!("\nExecuting chain with entity properties...");
                    let mut args_map = HashMap::new();

                    // Fill args from entity properties
                    for (k, v) in &seed.properties {
                        args_map.insert(k.clone(), *v);
                    }
                    // Add explicit CLI args (override entity properties)
                    for kv in &args {
                        args_map.insert(kv.key.clone(), kv.value);
                    }

                    let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                    match bankai.chain(&refs, &args_map) {
                        Ok(result) => {
                            if neutral {
                                let fmt = result.format();
                                let neutral_fmt: String = fmt
                                    .lines()
                                    .map(|line| {
                                        let mut l = line.to_string();
                                        for d in athena::wheel::ALL_DOMAINS.iter() {
                                            l = l.replace(d.symbol(), "");
                                            l = l.replace(d.full_name(), d.knowledge_domain());
                                        }
                                        l
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                println!("\n{}", neutral_fmt);
                            } else {
                                println!("\n{}", result.format());
                            }
                        }
                        Err(e) => {
                            eprintln!("Chain execution failed: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!(
                        "\nNo chain found to derive '{}' from entity '{}'s properties.",
                        want, entity
                    );
                    eprintln!("Try increasing --max-depth or adding bridging formulas.");
                }
            }
        }

        Commands::Pipeline { query, identity } => {
            let id = zanpakuto
                .authenticate(&format!("session_{}", identity))
                .unwrap_or(&default_identity);
            let pipeline = bankai.describe_pipeline(&query, id);
            println!("{pipeline}");

            // Show what Shikai would do (with NLP context)
            let nlp_ctx = nlp_engine.preprocess(&query);
            if let Ok(shikai_query) = shikai.process(&query, id, Some(&nlp_ctx)) {
                println!(
                    "\nShikai parsed:\n{}",
                    athena::shikai::Shikai::format_query(&shikai_query)
                );
                println!("\nNLP context:");
                println!("  Intent:    {:?}", nlp_ctx.likely_intent);
                if let Some(d) = nlp_ctx.likely_domain {
                    println!("  Domain:    {} ({})", d.symbol(), d.full_name());
                }
                if let Some(ref e) = nlp_ctx.likely_entity {
                    println!("  Entity:    {}", e);
                }
                println!("  QueryType: {:?}", nlp_ctx.query_type);
            }
        }

        // ─── Classification layer ──────────────────────────
        Commands::Classify { token } => {
            let sorter = athena::astrology::ChangeSorter::new();
            let classification = sorter.classify_token(&token);

            println!("=== Classification for '{}' ===", token);
            println!();

            // Dominant sign
            if let Some(sign) = classification.dominant_sign() {
                println!(
                    "  Sign:        {} {:?} ({:?})",
                    sign.symbol(),
                    sign,
                    sign.element()
                );
                // Show all sign activations
                println!("  All signs:");
                for i in 0..12 {
                    let s = athena::astrology::Sign::from_index(i);
                    let bar = "█".repeat((classification.signs[i] * 20.0) as usize);
                    println!(
                        "    {:12} {:3} {:20} {:.3}",
                        format!("{}{:?}", s.symbol(), s),
                        format!("[{}]", i),
                        bar,
                        classification.signs[i]
                    );
                }
            }

            println!();
            if let Some(element) = classification.dominant_element() {
                println!(
                    "  Element:     {} {:?} (strength: {:.3})",
                    element.symbol(),
                    element,
                    classification.elements[element.index()]
                );
            }
            if let Some(modality) = classification.dominant_modality() {
                println!(
                    "  Modality:    {} {:?} (strength: {:.3})",
                    modality.symbol(),
                    modality,
                    classification.modalities[modality.index()]
                );
            }
            println!(
                "  Polarity:    {:.3} (0=Yang, 1=Yin)",
                classification.polarity
            );

            println!();
            println!("  Vedic:");
            if let Some(graha) = classification.vedic.dominant_graha() {
                println!(
                    "    Graha:     {} {} ({})",
                    graha.symbol(),
                    graha.name(),
                    graha.element_affinity()
                );
            }
            if let Some(guna) = classification.vedic.dominant_guna() {
                println!(
                    "    Guṇa:      {} {} — {}",
                    guna.symbol(),
                    guna.name(),
                    guna.quality()
                );
            }
            if let Some(nak) = classification.vedic.dominant_nakshatra() {
                println!("    Nakṣatra:  {:?} — {}", nak, nak.shakti());
            }
            if let Some(ve) = classification.vedic.dominant_vedic_element() {
                println!(
                    "    Bhūta:     {} {} ({})",
                    ve.symbol(),
                    ve.sanskrit(),
                    ve.guna().quality()
                );
            }
        }

        Commands::Gyro => {
            let gyro = bankai.gyro_state();
            let (dominant_sign, dominant_mass) = gyro.dominant_sign_info();
            let weights = gyro.alignment_weights();

            println!("=== Gyroscopic Wheel State ===");
            println!();
            println!(
                "  Orientation:    {:.1}° ({:?})",
                gyro.orientation.0,
                gyro.orientation.dominant_sign()
            );
            println!(
                "  Dominant sign:  {:?} (mass: {:.3})",
                dominant_sign, dominant_mass
            );
            println!("  Angular vel:    {:.3} °/s", gyro.angular_velocity);
            println!("  Precession:     {:.3}", gyro.precession());
            println!("  Torque accum:   {:.6}", gyro.torque_accumulator);
            println!();
            println!("  Mass distribution:");
            for (i, &mass) in gyro.mass_distribution.iter().enumerate() {
                let s = athena::astrology::Sign::from_index(i);
                let bar = "█".repeat((mass * 20.0) as usize);
                println!(
                    "    {:12} {:20} {:.3}",
                    format!("{}{:?}", s.symbol(), s),
                    bar,
                    mass
                );
            }
            println!();
            println!("  Alignment weights:");
            for (i, &weight) in weights.iter().enumerate() {
                let s = athena::astrology::Sign::from_index(i);
                let bar = "█".repeat((weight * 20.0) as usize);
                println!(
                    "    {:12} {:20} {:.3}",
                    format!("{}{:?}", s.symbol(), s),
                    bar,
                    weight
                );
            }
        }

        // ─── Descent layer ─────────────────────────────────
        Commands::Descent { query } => {
            let matrix = descent_engine.descend(&query);
            println!("{}", matrix.format());
            println!("\n=== Vedic Summary ===");
            let agg = &matrix.aggregate_vedic;
            if let Some(graha) = agg.dominant_graha() {
                println!(
                    "  Graha:      {} {} ({})",
                    graha.symbol(),
                    graha.name(),
                    graha.element_affinity()
                );
            }
            if let Some(nak) = agg.dominant_nakshatra() {
                println!("  Nakṣatra:   {:?} — {}", nak, nak.shakti());
            }
            if let Some(guna) = agg.dominant_guna() {
                println!(
                    "  Guṇa:       {} {} — {}",
                    guna.symbol(),
                    guna.name(),
                    guna.quality()
                );
            }
            if let Some(ve) = agg.dominant_vedic_element() {
                println!(
                    "  Bhūta:      {} {} (Akasha: {})",
                    ve.symbol(),
                    ve.sanskrit(),
                    ve.sanskrit()
                );
            }
            println!("\n=== Descent Stats ===");
            println!("  Resolution:     {:.1}%", matrix.resolution_score * 100.0);
            println!("  Avg depth:      {:.2}/6", matrix.average_depth);
            println!("  Domains active: {}", matrix.dominant_domains.len());
            println!("  Aspects found:  {}", matrix.aspects.len());
        }

        Commands::DescentMatrix { query } => {
            let matrix = descent_engine.descend(&query);
            println!("{}", matrix.format());
        }

        // ─── Entity layer ──────────────────────────────────
        Commands::EntityList => {
            let seeds = entities.list_seeds();
            if seeds.is_empty() {
                println!("No seed entities loaded.");
            } else {
                println!("=== Seed Entities ({}) ===", seeds.len());
                for id in &seeds {
                    if let Some(s) = entities.get_seed(id) {
                        let sign_str = s
                            .classification
                            .as_ref()
                            .and_then(|c| c.dominant_sign())
                            .map(|sign| format!("{:?}", sign))
                            .unwrap_or_default();
                        println!("  {:30} | {:12} | {}", s.id, sign_str, s.description,);
                    }
                }
                let runtime_count = entities.len();
                if runtime_count > 0 {
                    println!(
                        "\n(+ {} runtime token entities — use `entity-get` for details)",
                        runtime_count
                    );
                }
            }
        }

        Commands::EntityGet { id } => {
            // Try seed entity first
            if let Some(s) = entities.get_seed(&id) {
                println!("=== Seed Entity: {} ===", s.name);
                println!("  ID:          {}", s.id);
                let sign_str = s
                    .classification
                    .as_ref()
                    .and_then(|c| c.dominant_sign())
                    .map(|sign| format!("{:?}", sign))
                    .unwrap_or_else(|| "none".to_string());
                println!("  Sign:        {}", sign_str);
                println!("  Description: {}", s.description);
                if !s.tags.is_empty() {
                    println!("  Tags:        {}", s.tags.join(", "));
                }
                if !s.properties.is_empty() {
                    println!("  Properties:");
                    for (k, v) in &s.properties {
                        println!("    {:<30} = {}", k, v);
                    }
                }
                if !s.constants.is_empty() {
                    println!("  Constants:");
                    for (k, v) in &s.constants {
                        println!("    {:<30} = {}", k, v);
                    }
                }
                if let Some(ref formula) = s.formula {
                    println!("  Formula:     {}", formula);
                }
            } else if let Some(e) = entities.get(&id) {
                println!("=== Runtime Entity: {} ===", e.text);
                println!("  ID:           {}", e.id);
                println!("  Text:         {}", e.text);
                println!("  Seq:          {}", e.seq);
                println!("  Truth:        {}", e.truth);
                if let Some(sign) = e.dominant_sign() {
                    println!("  Sign:         {:?}", sign);
                }
                if let Some(el) = e.dominant_element() {
                    println!("  Element:      {:?}", el);
                }
                if let Some(moda) = e.dominant_modality() {
                    println!("  Modality:     {:?}", moda);
                }
                if !e.tags.is_empty() {
                    println!("  Tags:         {}", e.tags.join(", "));
                }
                if !e.values.is_empty() {
                    println!("  Values:");
                    for (k, v) in &e.values {
                        println!("    {:<30} = {}", k, v);
                    }
                }
            } else {
                eprintln!("Entity not found: '{}'", id);
            }
        }

        Commands::EntityAspect { from, to } => match entities.aspect_between(&from, &to) {
            Some((aspect, a, b)) => {
                let sign_a = a
                    .dominant_sign()
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();
                let sign_b = b
                    .dominant_sign()
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();
                let aspect_desc = match aspect {
                    Aspect::Conjunction => "Conjunction — same sign, aligned",
                    Aspect::Sextile => "Sextile — adjacent, natural flow",
                    Aspect::Trine => "Trine — harmonious, complementary",
                    Aspect::Square => "Square — tension, requires work",
                    Aspect::Opposition => "Opposition — complementary opposites",
                };
                println!("=== Aspect: {} <-> {} ===", a.text, b.text);
                println!("  {} in {:?}", a.text, sign_a);
                println!("  {} in {:?}", b.text, sign_b);
                println!("  Aspect:      {:?} ({})", aspect, aspect_desc);
                println!(
                    "  Arc Distance: {} steps",
                    Aspect::arc_distance_between(
                        a.dominant_sign().map_or(0, |s| s.index()),
                        b.dominant_sign().map_or(0, |s| s.index()),
                    )
                );
            }
            None => eprintln!(
                "Cannot compute aspect: one or both entities not found ('{}', '{}')",
                from, to
            ),
        },

        Commands::EntitySearch { keyword } => {
            let seed_results = entities.search_seeds(&keyword);
            let runtime_results = entities.search(&keyword);
            if seed_results.is_empty() && runtime_results.is_empty() {
                println!("No entities found for '{}'.", keyword);
                // Also search formulas as a fallback
                let formulas = registry.search(&keyword);
                if !formulas.is_empty() {
                    println!("But found {} formula(s) with that keyword.", formulas.len());
                }
            } else {
                let total = seed_results.len() + runtime_results.len();
                println!("Found {} entity(ies) for '{}':", total, keyword);
                for s in &seed_results {
                    let sign_str = s
                        .classification
                        .as_ref()
                        .and_then(|c| c.dominant_sign())
                        .map(|sign| format!("{:?}", sign))
                        .unwrap_or_default();
                    println!("  {:30} | {:12} | {} [seed]", s.id, sign_str, s.description,);
                }
                for e in &runtime_results {
                    let sign_str = e
                        .dominant_sign()
                        .map(|s| format!("{:?}", s))
                        .unwrap_or_default();
                    println!("  {:30} | {:12} | {} [runtime]", e.id, sign_str, e.text,);
                }
            }
        }

        Commands::EntityEval {
            formula,
            entity,
            args,
        } => {
            let seed = match entities.get_seed(&entity) {
                Some(s) => s,
                None => {
                    eprintln!(
                        "Seed entity not found: '{}' (entity-eval requires a seed entity)",
                        entity
                    );
                    return Ok(());
                }
            };
            let mut args_map = HashMap::new();
            for kv in &args {
                args_map.insert(kv.key.clone(), kv.value);
            }
            // Fill missing args from seed entity properties first, then constants
            if let Some(f) = registry.get(&formula) {
                let entity_sign = seed
                    .classification
                    .as_ref()
                    .and_then(|c| c.dominant_sign())
                    .map(athena::wheel::Domain::from_sign);
                let domain_aligned = entity_sign == Some(f.domain);
                for input in &f.inputs {
                    if !args_map.contains_key(input) {
                        // Try properties first, then constants (with unit-suffix stripping)
                        let val = seed
                            .properties
                            .get(input)
                            .cloned()
                            .or_else(|| Bankai::match_constant(&seed.constants, input));
                        if let Some(v) = val {
                            args_map.insert(input.clone(), v);
                        }
                    }
                }
                // Warn if domain mismatch
                if !domain_aligned {
                    let seed_domain = entity_sign
                        .map(|d| d.full_name().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    println!(
                        "(Warning: formula '{}' is in {} but entity '{}' is in {} — domain mismatch)",
                        formula,
                        f.domain.full_name(),
                        entity,
                        seed_domain,
                    );
                }
            }
            // Evaluate the formula
            match bankai.evaluate(&formula, &args_map) {
                Ok(value) => {
                    println!("{} → {:.6}", formula, value);
                    println!("Entity: {} ({})", seed.name, entity);
                    println!("Args: {:?}", args_map);
                    // Show which args came from entity properties
                    let from_entity: Vec<String> = args_map
                        .keys()
                        .filter(|k| !args.iter().any(|a| a.key.as_str() == k.as_str()))
                        .cloned()
                        .collect();
                    if !from_entity.is_empty() {
                        println!(
                            "(Note: '{}' provided by entity '{}')",
                            from_entity.join(", "),
                            entity
                        );
                    }
                }
                Err(e) => eprintln!("Evaluation failed: {e}"),
            }
        }

        // ─── Budget layer ──────────────────────────────────
        #[cfg(feature = "budget")]
        Commands::Budget => {
            let stats = budget.stats();
            println!("{:?}", stats);

            // Also list recent token spends
            let spends = budget.spends();
            if spends.is_empty() {
                println!(
                    "\nNo token spends recorded yet. LLM calls are tracked here automatically."
                );
            } else {
                println!("\nRecent token spends:");
                for s in spends.iter().rev().take(10) {
                    println!(
                        "  {} | {} | {:>6} tok | {}",
                        s.id,
                        s.domain.symbol(),
                        s.total_tokens,
                        s.purpose,
                    );
                }
                // Total by domain
                let mut by_domain: std::collections::HashMap<String, (usize, usize)> =
                    std::collections::HashMap::new();
                for s in spends {
                    let entry = by_domain
                        .entry(s.domain.full_name().to_string())
                        .or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += s.total_tokens;
                }
                println!("\nTotal by domain:");
                let mut domains: Vec<_> = by_domain.into_iter().collect();
                domains.sort_by_key(|d| std::cmp::Reverse(d.1 .1));
                for (domain, (count, tokens)) in &domains {
                    println!("  {:12} | {:3} spends | {:6} tokens", domain, count, tokens);
                }
            }
        }

        #[cfg(feature = "budget")]
        Commands::BudgetReset => {
            budget.reset();
            println!("Token budget reset. All counters cleared.");
        }

        Commands::Ephemeris { date, time } => {
            let parts: Vec<i32> = date.split('-').filter_map(|p| p.parse().ok()).collect();
            let tparts: Vec<f64> = time.split(':').filter_map(|p| p.parse().ok()).collect();
            if parts.len() != 3 || tparts.is_empty() {
                eprintln!("Expected --date YYYY-MM-DD and --time HH:MM");
                std::process::exit(1);
            }
            let hour = tparts[0] + tparts.get(1).copied().unwrap_or(0.0) / 60.0;
            let jd = athena::ephemeris::julian_day(
                parts[0] as i16,
                parts[1] as u8,
                parts[2] as u8,
                hour,
            );
            let ayanamsa = athena::ephemeris::lahiri_ayanamsa(jd);
            println!("=== Ephemeris {date} {time} UT (JD {jd:.5}) ===");
            println!("Lahiri ayanamsa: {ayanamsa:.4}°  (sidereal = tropical − ayanamsa)\n");
            println!(
                "{:12} | {:>9} | {:>9} | {:10} | {:16} | pada",
                "graha", "tropical", "sidereal", "rashi", "nakshatra"
            );
            for pos in athena::ephemeris::all_graha_positions(jd) {
                println!(
                    "{:12} | {:8.4}° | {:8.4}° | {:10} | {:16} | {}",
                    pos.graha.full_name(),
                    pos.tropical,
                    pos.sidereal,
                    pos.rashi.name(),
                    pos.nakshatra.name(),
                    pos.pada
                );
            }
        }

        #[cfg(feature = "llm")]
        Commands::Distill { out, limit } => {
            let registry = build_default_registry();
            let entities = build_entity_registry();
            let mut pairs = athena::bankai::distill::distill(&registry, &entities);
            if let Some(n) = limit {
                pairs.truncate(n);
            }
            let mut buf = String::new();
            for p in &pairs {
                buf.push_str(&serde_json::to_string(p).expect("pair serializes"));
                buf.push('\n');
            }
            match out {
                Some(path) => {
                    if let Err(e) = std::fs::write(&path, &buf) {
                        eprintln!("distill: cannot write {}: {}", path, e);
                        std::process::exit(1);
                    }
                    eprintln!("Distilled {} training pairs -> {}", pairs.len(), path);
                }
                None => {
                    print!("{}", buf);
                    eprintln!("Distilled {} training pairs", pairs.len());
                }
            }
        }

        #[cfg(feature = "llm")]
        Commands::Llm {
            query,
            verbose,
            backend,
            model,
            endpoint,
            health,
            generate,
        } => {
            use athena::inference::{
                config::{ConfigOverrides, InferenceConfig},
                BackendKind,
            };
            use athena::llm::LlmRouter;
            use std::str::FromStr;

            // Build config overrides from CLI flags
            let overrides = ConfigOverrides {
                backend: backend
                    .as_deref()
                    .and_then(|s| BackendKind::from_str(s).ok()),
                model_path: model,
                endpoint_url: endpoint,
                temperature: None,
                max_tokens: None,
            };
            let config = InferenceConfig::load(overrides);

            let mut router = match LlmRouter::from_config(config) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("LLM router error: {}", e);
                    eprintln!();
                    eprintln!(
                        "Tip: Set ATHENA_MODEL_PATH or place qwen2.5-0.5b-instruct-q4_k_m.gguf"
                    );
                    eprintln!(
                        "     in ./models/, ~/.cache/athena/models/, or use --endpoint for remote"
                    );
                    std::process::exit(1);
                }
            };

            // Health check mode
            if health {
                let h = router.health();
                println!("=== Copilot Health ===");
                println!("Healthy:     {}", h.healthy);
                println!("Backend:     {}", h.backend_kind);
                println!("Model:       {}", h.model_loaded.unwrap_or_default());
                println!(
                    "Context:     {}",
                    h.context_size.map(|c| c.to_string()).unwrap_or_default()
                );
                println!("Message:     {}", h.message);
                println!("Capabilities:");
                for cap in router.capabilities() {
                    println!(
                        "  - {} v{}",
                        cap.name,
                        cap.version.as_deref().unwrap_or("?")
                    );
                    for feat in &cap.supported_features {
                        println!("      feat: {}", feat);
                    }
                }
                return Ok(());
            }

            // Generation mode (free-form)
            if generate {
                match router.generate(&query, None) {
                    Ok(resp) => {
                        println!("{}", resp.text);
                        if verbose {
                            eprintln!(
                                "--- tokens: {}, finish: {} ---",
                                resp.tokens_generated, resp.finish_reason
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Generation failed: {}", e);
                        std::process::exit(1);
                    }
                }
                return Ok(());
            }

            // Routing mode (default)
            match router.route(&query) {
                Ok(decision) => {
                    if verbose {
                        println!("=== LLM Routing Decision ===");
                        println!("Query:        {}", query);
                        println!("Category:     {:?}", decision.category);
                        println!("Confidence:   {:.2}", decision.confidence);
                        println!("Deterministic: {}", decision.deterministic_only);
                        println!("Tokens:       {}", decision.tokens.len());
                        for (i, t) in decision.tokens.iter().enumerate() {
                            println!(
                                "  [{i}] text={} domain={:?} mass={}",
                                t.text, t.domain, t.mass
                            );
                        }
                        println!("Suggested chain: {:?}", decision.suggested_chain);
                        println!("Suggested entities: {:?}", decision.suggested_entities);
                        println!("Explanation:");
                        println!("  {}", decision.explanation);
                    } else {
                        println!(
                            "category: {:?} | confidence: {:.2} | chain: {:?}",
                            decision.category, decision.confidence, decision.suggested_chain
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Routing failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Mcp => {
            #[cfg(feature = "mcp")]
            {
                let athena_mcp = athena::mcp::AthenaMCP::with_entities(registry, entities.clone());
                let handler = athena::mcp::McpHandler::new(athena_mcp);
                eprintln!("Athena MCP server starting (stdio)...");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // serve_server resolves once the initialize handshake is
                    // done; the returned service must be awaited or the
                    // server exits before handling a single tool call.
                    match rmcp::serve_server(handler, (tokio::io::stdin(), tokio::io::stdout()))
                        .await
                    {
                        Ok(service) => {
                            if let Err(e) = service.waiting().await {
                                eprintln!("MCP server error: {e}");
                            }
                        }
                        Err(e) => eprintln!("MCP server error: {e}"),
                    }
                });
            }
            #[cfg(not(feature = "mcp"))]
            {
                eprintln!("MCP server not available (compile with --features mcp)");
            }
        }
    }
    Ok(())
}

// ─── Output helpers ────────────────────────────────────────────────────────

fn print_validation(result: &GateResult) {
    println!("=== Validation Result ===");
    for gate in &result.gates {
        let icon = if gate.passed { "✓" } else { "✗" };
        println!(
            "  {icon} {}: {} (confidence: {:.2})",
            gate.gate, gate.message, gate.confidence
        );
        for issue in &gate.issues {
            println!("    Issue: {issue}");
        }
        for suggestion in &gate.suggestions {
            println!("    → {suggestion}");
        }
    }
}

fn print_solve(solve: &athena::bankai::BankaiSolve) {
    println!("=== Bankai Solve ===");
    println!("{}", solve.summary);
    if let Some(chain) = &solve.chain {
        println!("\n{}", chain.format());
    }
    if let Some(validation) = &solve.validation {
        println!("\nValidation: {}", validation.summary);
    }
}

// ─── Default Formula Registry ──────────────────────────────────────────────

fn build_default_registry() -> FormulaRegistry {
    let mut registry = FormulaRegistry::new();
    let mut file_count = 0u32;

    // Load primitive formulas from formulas/atomic/ and cross-domain
    // formulas from formulas/bridging/ (previously only atomic was loaded,
    // leaving the bridging files as dead data).
    for dir in ["formulas/atomic", "formulas/bridging"] {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "toml") {
                        match registry.load_from_file(&path) {
                            Ok(()) => file_count += 1,
                            Err(e) => eprintln!("Warning: {}", e),
                        }
                    }
                }
            }
            Err(e) => eprintln!("Warning: formula directory '{}/' — {}", dir, e),
        }
    }

    // Rebuild TF-IDF index once after all formulas are loaded
    registry.rebuild_tfidf();

    eprintln!(
        "Athena: loaded {} primitive formulas from {} file(s)",
        registry.len(),
        file_count,
    );

    registry
}

fn build_entity_registry() -> EntityRegistry {
    let mut entity_registry = EntityRegistry::new();
    let mut file_count = 0u32;

    let entity_dirs = ["entities"];

    for dir in &entity_dirs {
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "toml") {
                        match entity_registry.load_seeds_from_file(&path) {
                            Ok(()) => file_count += 1,
                            Err(e) => eprintln!("Warning: {}", e),
                        }
                    }
                }
            }
            Err(e) => eprintln!("Warning: entity directory '{}' — {}", dir, e),
        }
    }

    eprintln!(
        "Athena: loaded {} seed entities from {} file(s)",
        entity_registry.seed_count(),
        file_count
    );

    entity_registry
}
