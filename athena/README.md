<div align="center">

# Ł L.ai · *Athena*

**Relational intelligence — formulas, not facts.**

> Part of the **[L.ai](https://github.com/nutypebuddha/lai)** ecosystem — *verify, don't trust.*

<img src="../docs/brand-LAI.svg" width="72" height="96" alt="L.ai witch-hat Ł glyph"/>

</div>

---

**Relational intelligence — formulas, not facts.**

Athena is a deterministic cross-domain reasoning engine in the L.ai family. It stores no static
facts — instead, it encodes the *relational machinery* that connects concepts across domains.
Given a question, Athena traverses its symbolic graph of formulas to find, compose, and validate
reasoning chains.

⚠️ **PROPRIETARY SOFTWARE** — Licensed under Athena Proprietary License. See LICENSE for terms.
Unauthorized copying, distribution, or reverse engineering is strictly prohibited.

Born from CID — whose Tanto engine handles single-domain math verification — Athena is the released form: *Bankai*.

---

## Core Concept

Most knowledge systems store facts: `c = 299,792,458 m/s`. LLMs already have those facts. Athena
stores *formulas* — the relationships *between* facts:

- `mass * acceleration -> force` (Taurus/Physics)
- `force = mass * velocity / time -> momentum` (Aries/Math)
- `momentum -> kinetic_energy` (Aries -> Taurus, Bridging)

The 12 zodiac wheel nodes form a symbolic graph. Formulas live at the nodes, and traversing the
wheel's edges produces *chains* of reasoning across domains. The value is in the chain, not in any
single node.

---

## Architecture

Athena has a **4-layer** architecture inspired by the Bleach concept of ascending power levels:

```
                    ┌──────────────────────┐
                    │     Asauchi CLI      │  Public interface
                    │   (info, validate)   │
                    ├──────────────────────┤
                    │     Zanpakuto        │  Identity & auth
                    │   (register, tier)   │
                    ├──────────────────────┤
                    │     Shikai           │  NL query parser
                    │ (intent, formulas,   │  intent, domain,
                    │  args, entities)     │  entity + formula
                    │                      │  resolution
                    ├──────────────────────┤
                    │     Bankai           │  Computation engine
                    │  eval │ chain │      │  evaluate, chain,
                    │  compose │ traverse  │  compose, traverse
                    │  solve │ search      │
                    ├──────────────────────┤
                    │  Entity Registry     │  65 grounded entities
                    │   + Token Budget     │  across 12 domains,
                    │                      │  every token tracked
                    ├──────────────────────┤
                    │   Zodiac Wheel       │  12 domains x 5
                    │   Graph + Registry   │  aspects, 63 formulas
                    ├──────────────────────┤
                    │  Validation Gates    │  math, logic, formal,
                    │                      │  confidence, fact
                    └──────────────────────┘
                    │    MCP Server        │  stdio JSON-RPC
                    │  (validate, traverse,│  entity tools,
                    │   compose, search,   │  budget tools)
                    └──────────────────────┘
```

### Layer Descriptions

| Layer | Name | Purpose |
|-------|------|---------|
| 1 | **Asauchi** | Public CLI entry point (info, validate, ping). The nameless blade. |
| 2 | **Zanpakuto** | Identity management, authentication, and access tier control (Shikai/Bankai). |
| 3 | **Shikai** | Natural language query parser — extracts intent (evaluate, info, search, traverse, validate), domains, formula IDs, and argument values from raw input. |
| 4 | **Bankai** | The core reasoning engine — evaluates formulas, chains them sequentially, composes cross-domain bridges, traverses the wheel, and solves complete queries. |

---

## Formula Tiers

| Tier | Scope | Traversal Depth | Example |
|------|-------|----------------|---------|
| **Atomic** | Single domain | 0 hops | `F = ma` (Taurus / Physics) |
| **Bridging** | 2 domains | 1 hop | `momentum -> kinetic_energy` (Aries -> Taurus) |
| **Vortex** | 4+ domains in a spiral | 3+ hops | Philosophy -> Language -> CS -> Math -> Psychology |

---

## The Zodiac Wheel

The 12 domains are arranged as a symbolic wheel. Signs are *nominal labels* for domain archetypes,
not literal astrological positions. Relationships between domains are defined by their *aspect*:

| Node | Domain | Symbol |
|------|--------|--------|
| Aries | Math & Logic | ARI |
| Taurus | Physics & Chemistry | TAU |
| Gemini | Astronomy & Cosmology | GEM |
| Cancer | Earth & Environment | CAN |
| Leo | Biology & Medicine | LEO |
| Virgo | Economics & Finance | VIR |
| Libra | Engineering & Tech | LIB |
| Scorpio | Computer Science & AI | SCO |
| Sagittarius | History & Anthropology | SAG |
| Capricorn | Language & Linguistics | CAP |
| Aquarius | Philosophy & Ethics | AQU |
| Pisces | Psychology & Neuroscience | PIS |

Aspects (edge types): **Conjunction** (self), **Sextile** (adjacent, 1 step), **Trine** (harmonious, 4 steps),
**Square** (tension, 3 steps), **Opposition** (complementary, 6 steps).

---

## Quick Start

```bash
# Build
cargo build --release

# Show system info
athena info

# Display the zodiac wheel
athena wheel

# Show a domain's connections
athena wheel --domain taurus

# Search formulas by keyword
athena search momentum

# Evaluate a formula with arguments
athena eval --formula newtons_second --args mass=5 --args acceleration=9.8
# -> 49.0 (F = 5 * 9.8)

# Chain formulas across domains
athena chain --formulas "newtons_second,momentum_to_ke" --args mass=5 --args acceleration=9.8 --args velocity=10
# -> ke = 250.0

# Compose formulas into a reasoning chain
athena compose --formulas "pythagorean,newtons_second"
# -> Bridge: Aries -> Taurus, confidence 0.95

# Traverse the wheel from a domain
athena traverse --domain taurus --depth 3

# Solve a query using natural language
athena solve "newtons_second mass=5 acceleration=9.8"

# Validate an expression
athena validate "2 + 2" --gate math

# See how a query is parsed (Shikai layer)
athena shikai "calculate force mass=5 acceleration=9.8"

# Show the full pipeline for a query
athena pipeline "calculate force mass=5 acceleration=9.8"

# List all entities (knowledge graph subjects)
athena entity-list

# Get entity details
athena entity-get --id major_depression

# Compute aspect between two entities
athena entity-aspect --from schizoaffective_disorder --to lithium

# Search entities by keyword
athena entity-search mood

# Evaluate a formula grounded in an entity (auto-fills args from entity properties)
athena entity-eval --formula arousal_valence --entity major_depression
# -> -0.16 (valence=-0.8, arousal=0.2 from entity properties)

# Show token budget usage
athena budget

# Reset token budget
athena budget-reset

# Entity-aware Shikai: entity names are auto-resolved,
# missing formula args filled from entity properties
athena shikai "solve major_depression"
# -> Intent: Evaluate, Domain: Pisces
# -> Formula: arousal_valence (auto-discovered via property overlap)
# -> Args: [arousal=0.2, valence=-0.8] (auto-filled from entity)

# MCP server (stdio JSON-RPC, includes entity + budget tools)
athena mcp
```

---

## Formula Database

Formulas are stored as TOML files in `formulas/` and loaded at runtime. No recompilation needed.

### Directory Layout

```
formulas/
  atomic/
    math.toml
    physics.toml
    astronomy.toml
    earth.toml
    biology.toml
    cs.toml
    engineering.toml
    economics.toml
    history.toml
    language.toml
    philosophy.toml
    psychology.toml
  bridging/
    matter_energy.toml
    systems.toml
    knowledge.toml
    structure.toml
  vortex/
    spirals.toml
  nonmath/
    grammar.toml
    code.toml
    logic.toml
```

### Atomic Formula (TOML)

```toml
[[formula]]
id = "newtons_second"
domain = "taurus"
inputs = ["mass", "acceleration"]
output = "force"
expression = "mass * acceleration"
description = "Newton's second law: F = ma"
```

### Bridging Formula (TOML)

```toml
[[formula]]
id = "momentum_to_ke"
from = "aries"
to = "taurus"
inputs = ["mass", "velocity"]
output = "ke"
expression = "0.5 * mass * velocity^2"
description = "Momentum to kinetic energy"
aspect = "trine"
```

### Vortex Spiral (TOML)

```toml
[[spiral]]
id = "scientific_method"
name = "The Scientific Method"
description = "Observation -> Hypothesis -> Experiment -> Theory -> Prediction"
domains = ["pisces", "leo", "taurus", "aries"]

[[spiral.steps]]
from = "pisces"
to = "leo"
aspect = "sextile"
formula_in = "observation"
formula_out = "hypothesis"

[[spiral.steps]]
from = "leo"
to = "taurus"
aspect = "square"
formula_in = "hypothesis"
formula_out = "experiment"
```

---

## Project Status

All components are built, tested, and operational:

| Component | Status | Lines |
|-----------|--------|-------|
| Wheel Graph | Complete | ~600 |
| Formula System | Complete | ~500 |
| Formula Database | Complete (63 formulas) | ~600 (TOML) |
| Entity Registry | Complete (65 entities) | ~900 (TOML + Rust) |
| Token Budget | Complete | ~400 |
| Bankai Engine | Complete | ~600 |
| Validation Gates | Complete | ~400 |
| Shikai Parser | Complete | ~400 |
| Zanpakuto Auth | Complete | ~150 |
| MCP Server | Complete | ~400 |
| CLI | Complete | ~750 |

**Tests**: 149 passing (114 unit + 35 integration), 0 warnings, 0 errors.

---

## Entity Knowledge Graph

Athena's entity system grounds the abstract zodiac wheel in concrete referents. Each entity
is a named thing placed on a domain with its own numeric properties.

### Entity File (TOML)

```toml
# entities/pisces.toml
[[entity]]
id = "major_depression"
name = "Major Depressive Disorder"
domain = "pisces"
description = "DSM-5 diagnosis: persistent depressed mood, anhedonia, cognitive impairment"
tags = ["psychiatry", "mood", "depression", "DSM-5"]
properties = { mood_instability = 0.4, valence = -0.8, arousal = 0.2, affective_intensity = 0.6 }
```

### Entity Resolution in Shikai

When a query mentions an entity name, Shikai automatically:

1. **Resolves the entity** by matching words against the entity registry
2. **Sets the query domain** to the entity's domain (if no domain explicitly given)
3. **Discovers relevant formulas** by matching entity property names against formula inputs
4. **Fills missing formula arguments** from entity property values

```bash
# Before: had to specify domain, formula, and args manually
athena solve "arousal_valence for major_depression"

# After: entity-aware, just name the entity
athena solve "solve major_depression"
# → auto-discovers arousal_valence (properties match formula inputs)
# → auto-fills arousal=0.2, valence=-0.8 from entity properties
# → evaluates to -0.16
```

### Entity-to-Entity Aspect

Entities on different domains have aspect relationships defined by the zodiac wheel.
The aspect represents the nature of the relationship between two entities:

```bash
athena entity-aspect --from schizoaffective_disorder --to lithium
# → Sextile (adjacent, natural flow), confidence 0.85
```

### Current Entities (65 across 12 domains)

| Domain | Entities | Examples |
|--------|----------|---------|
| ♈ Aries (Math & Logic) | 6 | bayesian_inference, signal_detection_theory, falsification |
| ♉ Taurus (Physics & Chem) | 6 | lithium, ssri, antipsychotic, neurotransmitter |
| ♊ Gemini (Astronomy & Comm) | 4 | mercury, cognitive_flexibility, theory_of_mind |
| ♋ Cancer (Earth & Env) | 4 | urban_mental_health, climate_anxiety, environmental_enrichment |
| ♌ Leo (Biology & Med) | 8 | prefrontal_cortex, amygdala, hippocampus, neuroplasticity |
| ♍ Virgo (Economics) | 4 | mental_health_economics, pharmacoeconomics |
| ♎ Libra (Engineering) | 5 | fmri, eeg, digital_phenotyping, computational_psychiatry |
| ♏ Scorpio (CS & AI) | 6 | diagnostic_classifier, graph_neural_network, explainable_AI |
| ♐ Sagittarius (History) | 6 | dsm_history, psychopharmacology_history, freudian_psychoanalysis |
| ♑ Capricorn (Language) | 4 | clinical_interview, diagnostic_formulation |
| ♒ Aquarius (Philosophy) | 4 | mind_body_problem, qualia, medical_ethics |
| ♓ Pisces (Psychology) | 8 | schizoaffective_disorder, major_depression, dopamine |

---

## Token Budget

Athena tracks every LLM token consumed as an **entity** on the wheel. The budget is not an external
counter — it's a query over the entity registry:

```bash
# Show current usage
athena budget
# → Token Budget: 0% used (0/3000 total)

# Reset counters
athena budget-reset
```

Features:
- Every `TokenSpend` is an `Entity` tagged with `token_spend`, domain, purpose, and source
- Hard caps: prompt (2000), completion (1000), total (3000) tokens per session
- `BudgetCheck::Ok/Warning/Blocked` enforcement before LLM calls
- Deterministic fallback on budget exceeded (no silent partial spends)
- Per-domain breakdown of token usage

---

## MCP Server Tools

Athena exposes its capabilities as MCP tools for LLM integration:

| Tool | Description |
|------|-------------|
| `athena_validate` | Validate a claim through math/logic/formal gates |
| `athena_traverse` | Traverse the zodiac wheel from a domain |
| `athena_compose` | Compose formulas into a reasoning chain |
| `athena_formula_search` | Search formulas by keyword |
| `athena_wheel` | Display wheel graph structure |
| `athena_entity_list` | List all entities in the knowledge graph |
| `athena_entity_get` | Get entity details by ID |
| `athena_entity_aspect` | Compute aspect between two entities |
| `athena_entity_search` | Search entities by keyword |
| `athena_budget_stats` | Show token budget usage |

---

## Non-Math Formulas

Athena also supports grammar rules, code patterns, and logical structures as first-class formulas.
These are loaded from `formulas/nonmath/` alongside the mathematical formulas.

```toml
[[logic]]
id = "modus_ponens"
form = "If P then Q. P. Therefore Q."
premises = ["If P then Q", "P"]
conclusion = "Q"
is_valid = true

[[grammar]]
id = "subject_verb_agreement"
pattern = "^(\\w+) (\\w+) the (\\w+)$"
transform = "$1 $2s the $3"
description = "Basic subject-verb agreement"
rule_type = "prescriptive"

[[code]]
id = "early_return"
language = "general"
template = "if (condition) { return value; }"
alternative = "Single exit point pattern"
complexity = 3
is_anti_pattern = false
```

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

*Born from CID. Released as Bankai.*
