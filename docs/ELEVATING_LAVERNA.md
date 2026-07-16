# Elevating Laverna — A Comprehensive Strategic & Technical Roadmap

> **Note:** "Laverna" is the internal code name. The public mark is **L.ai**
> (see `docs/brand.md`); *Laverna* = **L.ai · Proof**. This document is the
> historical strategic roadmap and retains the code name throughout.

> **Subject:** `laverna-x86_64` v0.1.0 — "Deterministic Vedic reasoning engine — 9-graha wheel, NAND-to-bankai compute" (Rust, edition 2021)
> **Repo:** https://github.com/nutypebuddha/Laverna (MIT, created 2026-07-12, 5 open issues, 0 stars at research time)
> **Date of this document:** 2026-07-14
> **Method:** Black-box CLI/MCP testing of the shipped binary + live web research on the surrounding ecosystem (Vedākṣha, XALEN, MCP spec, VSOP87, World Bank API, deterministic-reasoning peers, Rust CLI conventions, MILP solvers).

---

## 0. TL;DR — The strategic posture in one paragraph

Laverna is **genuinely original** — no other project fuses Vedic Navagraha classification, NAND-completeness-as-proof, a 7-layer descent cascade (Macro → Micro → Mesa → Element → Compound → Nand → Sheffer), and a Bleach-themed pipeline (Asauchi → Zanpakutō → Shikai → Bankai) into one deterministic Rust binary. Its "never guesses, fails loud" philosophy is philosophically defensible and architecturally clean. **But it is also two days old, 0-starred, has 5 open issues, runs an MCP server 3 spec revisions behind, ships two reproducible bugs in `websearch`, has a stale single-ayanamsa ephemeris (probably Moon-accuracy-limited), exposes only 7 of 14 subcommands over MCP, reinvents a small Pareto solver that won't scale, and lacks any corpus-versioning or community-extension surface.** The window to elevate Laverna from "interesting personal project" to "category-defining tool" is open **right now** — but Vedākṣha (12-tool MCP, property-graph, 44 ayanamshas, BSL-1.1) and XALEN (deepest pure-Rust ephemeris, Apache-2.0) are already in the space and will out-feature it within 6–12 months unless Laverna picks its battles.

**The thesis of this document:** Laverna's defensible differentiator is **NOT** the Vedic chart (Vedākṣha and XALEN both beat it there) — it is the **NAND-to-bankai proof cascade** plus the **"fail-loud, no-hallucination" contract** plus the **deterministic 7-pillar optimization**. Every elevation move should pour concrete on *those three pillars* and treat the astrology layer as a *profile*, not the product.

---

## 1. What Laverna actually is — observed architecture (from binary probing)

The README at the public repo confirms what black-box testing inferred. Laverna is layered:

```
Layer 3 — verify       Expression verification, diagnostics, confidence scoring, feedback protocol
Layer 2 — nlp/query    NLP tokenization, intent parsing, domain classification
Layer 1 — aspect       Formula registry, entity registry, ephemeris, charts
Layer 0 — primitive    NAND gates, descent engine, router (the 9-graha wheel)
```

Pipeline: `query → nlp_parse → descent_engine → query_process → verify_solve`

### 1.1 The 7-layer descent cascade (concrete, observed)

Black-box testing of `solve` shows tokens settle at one of seven layers, from shallow (d0) to deep (d6):

| Layer | Name      | Observed settlement | Semantics |
|-------|-----------|---------------------|-----------|
| d0    | Macro     | "Does", "bipolar"   | Token recognized but unresolved — could not bind to a formula or entity. Counts toward 0% resolution. |
| d1    | Micro     | (not directly observed in test queries) | Likely: token classified into a domain but no formula matched. |
| d2    | Mesa      | (not directly observed) | Likely: domain bound, but unification with entity failed. |
| d3    | Element   | "reuptake"           | Partial: domain + formula matched, entity bound, but not reduced to NAND. |
| d4    | Compound  | (not directly observed) | Likely: formula evaluated but result not yet provable down to NAND. |
| d5    | Nand      | "2", "3", "5", "serotonin", "mood", ... | Full reduction: token has domain + formula + entity + NAND-absolute classification. |
| d6    | Sheffer   | (referenced in binary, not surfaced as a settlement layer in user output) | Deepest layer — likely the "NAND-as-primitive" proof object itself. |

> **Note:** the user-visible `solve` output prints settlement as `NAND (d6)` for fully-resolved tokens, which suggests **Nand is the user-facing "deepest" layer**; **Sheffer** appears to be the internal primitive name (the Sheffer stroke = NAND). The 7-layer naming in the binary (Macro/Micro/Mesa/Element/Compound/Nand/Sheffer) maps to the d0–d6 numbers visible in `--verbose`/`--explain` output.

### 1.2 The 9-graha wheel and the 7 pillars

- **9 grahas** (Navagraha): Surya, Chandra, Mangala, Budha, Brihaspati, Shukra, Shani, Rahu, Ketu. All 9 are used for token classification and routing.
- **7 pillars**: Spear (Surya), Olive (Chandra), Forge (Mangala), Owl (Budha), Council (Brihaspati), Loom (Shukra), Stone (Shani). **Rahu and Ketu are excluded** — consistent with classical Jyotish (they are lunar nodes, not true planetary bodies).
- **Pillar taxonomy is original** — no public source for the Spear/Olive/Forge/Owl/Council/Loom/Stone naming was found. Each name evokes its graha (Forge↔Mars/fire/metalwork, Owl↔Mercury/intellect, Stone↔Saturn/lead/endurance, etc.).
- **Lagna archetypes** (16 observed in binary strings): Stride, Cut, Threshold, Trail, Renewal, Bread, Crown, Repose, Vow, Gem, Target, Altar, Elder, Spear, Foundation, Shield, Acropolis, Ear, Rhythm, Remedy, Blaze, Depths, Guide. These pair with the 12 rashis to produce the chart's `personality.archetype` field (e.g. "Depths" for Simha lagna in our test chart).

### 1.3 Embedded corpus

- **528 formulas**, each tagged with exactly one primary graha (Surya/Budha/Chandra/Ketu/Mangala/Rahu/Shani/Shukra/Brihaspati).
- **214 entities**, distributed across grahas (Surya 86, Brihaspati 25, Mangala 18, Shukra 15, Budha 15, Shani 14, Rahu 11, Ketu 11, Chandra 9, 10 ungrouped).
- Built-in `build.rs` embeds the corpus at compile time. No external data files needed at runtime.
- 27 nakshatras (Ashwini through Revati) — confirmed via strings extraction.
- 12 rashis (Mesha through Meena).
- **Embedded proof of NAND completeness via Post's criterion** (found in binary strings):
  > `premises = ["NAND is not affine (depends on both inputs non-trivially)", "NAND is not self-dual", "NAND is not monotone", "Post's criterion: a gate is functionally complete iff it is not affine, not self-dual, not monotone, not the constant 0, not the constant 1"]`

  This is mathematically correct and **deserves to be a first-class, citable artifact** — currently it's hidden inside a stripped binary.

### 1.4 Feature flags (confirmed from README)

| Flag       | Status | Purpose |
|------------|--------|---------|
| `mcp`      | compiled in default binary | rmcp + tokio JSON-RPC over stdin/stdout |
| `websearch`| compiled in default binary | ureq → World Bank API |
| `budget`   | compiled in default binary | budget-constrained optimization |
| `bench`    | NOT exposed as subcommand | criterion-based benchmarks (compile-time only) |
| `llm`      | NOT exposed as subcommand | local llama-gguf backend — present in source but no user-facing subcommand |

> **Hidden surface:** the `llm` and `bench` features exist but are inaccessible from the CLI. Either they're scaffolding for future releases, or they were forgotten during the v0.1.0 cut. **This is a v0.2 opportunity.**

### 1.5 Bugs confirmed during testing (these block adoption)

| # | Bug | Severity | Root cause (inferred) |
|---|-----|----------|-----------------------|
| 1 | `schema optimize` / `schema domain` print `eta]` instead of `[meta]` as the first non-comment line | **High** (blocks first-run) | Off-by-one or column-counting bug in the schema-template formatter. Anyone copying the template verbatim gets `missing field meta`. |
| 2 | `entities` formatter drops the leading `[` + first 1–2 chars of the property list when it wraps onto the description's line (e.g. `alf_life_hours` instead of `[half_life_hours`) | Medium (UX) | String-width / wrap calculator not accounting for the `[` prefix. |
| 3 | `chart` lagna line prints `Simha (Simha)` — duplicated name where English name ("Leo") or symbol ("♌") is expected | Low (cosmetic) | Format string template bug. |
| 4 | `route` silently ignores `--query` when `--repos` is supplied | Low (UX) | Should warn. |
| 5 | `websearch` mis-segments compound queries (`"CO2 emissions China 2018"` → country `"EMISSIONS CHINA"`) | High (functional) | `zanpakuto_nlp` layer has no ISO-3166 country gazetteer; it greedily maps free text to country slot. |
| 6 | `websearch` times out on multi-year queries without an explicit year | High (functional) | Likely fires one HTTP request per year sequentially instead of using the World Bank `date=YYYY:YYYY` range form. |
| 7 | MCP server is 3 spec revisions behind (2024-11-05 → 2025-11-25) | Medium (ecosystem) | Spec has moved on; missing tool annotations, structured output, elicitation, resource links, `title` field, icons, tasks, JSON-Schema-2020-12. |
| 8 | Only 7 of 14 CLI subcommands are exposed as MCP tools (`route`, `build`, `websearch`, `ping`, `info`, `schema` are missing) | Medium (coverage) | Should expose `route` and `build` at minimum; `formulas`/`entities`/`schema` should arguably be Resources, not Tools. |

These are all **fixable in one focused week**. None of them are architectural.

---

## 2. The competitive landscape — and where Laverna actually wins

### 2.1 The two real competitors

| Dimension | **Laverna** | **Vedākṣha** | **XALEN** |
|-----------|-------------|--------------|-----------|
| URL | github.com/nutypebuddha/Laverna | github.com/arthiqlabs/vedaksha | github.com/vedika-io/xalen-ephemeris |
| License | **MIT** (truly OSS) | BSL-1.1 → Apache after 5yr (source-available, non-commercial) | **Apache-2.0** (truly OSS) |
| Focus | Vedic *reasoning* + NAND proof | Vedic *computation* + property graph | *Ephemeris* engine (multi-tradition) |
| Stars | 0 | larger | larger |
| MCP tools | 7 | 12 | n/a (library) |
| Property graph | No | **Yes (Cypher/SurrealQL/JSON-LD)** | No |
| Ephemeris | Self-rolled VSOP87 + Lahiri (1 ayanamsa) | VSOP87A + ELP/MPP02 + DE440 (44 ayanamshas) | VSOP87A + ELP2000-82 + DE440 (50 ayanamshas, **deepest**) |
| Moon accuracy | **Probably poor** (VSOP87 alone) | Sub-arcsecond (ELP/MPP02) | RMS ~2.8″ vs pyswisseph |
| Corpus | 528 formulas + 214 entities | 870 tests + 8700 oracle rows | 506 fixed stars + 8870 Hipparcos |
| Determinism claim | **Explicit, central** (NAND, fail-loud) | Implicit (clean-room, cited) | n/a (library) |
| ML | Banned in core (optional `llm` flag, unused) | None | None |
| Bleach motif | Pervasive | None | None |
| Bindings | Rust only (4 musl/gnu targets) | Rust/Python/WASM/Docker | Rust/Python/Node/WASM/C |
| Maturity | v0.1.0, 2 days old | v3.2.0, established | v0.3.1 published, 0.5.x WIP |

### 2.2 The honest assessment

Laverna **cannot win** on:
- **Ephemeris depth** — XALEN has 50 ayanamshas and a sub-arcsecond Moon; Laverna has 1 ayanamsa and probably a degraded Moon. This is a 6-month gap to close if Laverna wanted to compete here, and it's not worth closing.
- **MCP tool surface area** — Vedākṣha has 12 tools to Laverna's 7, including `compute_dasha`, `compute_vargas`, `compute_karakas`, `compute_shadbala`, `compute_ashtakavarga`, `compute_transit`, `compute_gochara`, `search_transits`, `search_muhurta`. These are core Jyotish computations Laverna doesn't expose at all.
- **Property-graph output** — Vedākṣha emits Cypher/SurrealQL/JSON-LD; Laverna emits flat JSON. For agentic workflows that want to *query* a chart ("which grahas aspect the 7th house lord?"), property-graph is the right shape.
- **Multi-language bindings** — Vedākṣha and XALEN both ship Python/WASM; Laverna is Rust-only.

Laverna **can win** on:
- **Deterministic reasoning contract** — no competitor makes "never guesses, fails loud" a central, architectural commitment. This is Laverna's most differentiated asset.
- **NAND-to-bankai proof cascade** — entirely original. No competitor has a 7-layer descent terminating at a formally-complete primitive.
- **Embedded formula corpus** — 528 cross-domain formulas, each tagged to a graha, is a unique KR asset. Vedākṣha and XALEN don't have anything equivalent.
- **Optimization layer** — `optimize` and `build` (chart → graha weights → Pareto-optimal allocation) is unique. Neither competitor offers this.
- **MIT license** — truly OSS, vs Vedākṣha's BSL-1.1. For adoption in commercial products, this matters.
- **Bleach-themed narrative** — memorable, brandable, community-building. Sounds silly as a "feature" but it's a real differentiator for a young project trying to attract contributors.

### 2.3 The strategic implication

Laverna should **stop trying to be a better Vedic astrology engine**. It should **be the best deterministic reasoning engine that happens to use Vedic astrology as one of its classification profiles**. Concretely:
- The `chart` subcommand should **delegate ephemeris computation to XALEN** (Apache-2.0, MIT-compatible) instead of self-rolling VSOP87.
- The 9-graha wheel should be repositioned as **one of several possible "classifier profiles"** — the same descent cascade could work with a 5-element (Wu Xing) profile, a 4-temperament profile, a 16-MBTI profile, etc.
- The `optimize` + `build` pipeline should be the **flagship product**.
- The NAND-to-bankai cascade should be **externally auditable** — every `solve` should optionally emit a proof object that can be re-verified by a separate `verify` command.

---

## 3. The 8-dimension elevation plan

| # | Dimension | One-line thesis |
|---|-----------|-----------------|
| A | **Architecture & module boundaries** | Externalize the ephemeris; clarify the wheel-vs-engine distinction; make the descent cascade auditable. |
| B | **Corpus & knowledge representation** | Version the corpus independently; graph-ify the formulas; open a community-extension surface. |
| C | **Solver & optimization** | Keep exact-Pareto as default; add optional MILP backend for scale; expose multi-objective sweeps. |
| D | **MCP compliance & agentic surface** | Bump to 2025-11-25; expose `route`/`build`/`websearch`; convert discovery surfaces to Resources/Prompts. |
| E | **Web integration (World Bank + beyond)** | Add ISO-3166 gazetteer; use range queries; add FRED/IMF/OECD backends; cache aggressively. |
| F | **UX, CLI ergonomics & docs** | Fix the 3 cosmetic bugs in a day; add `--json` everywhere; ship a `laverna repl`; write a real docs site. |
| G | **Determinism, proof objects & verification** | Emit Lean-style proof objects; add a `verify` subcommand; separate proof search from proof checking. |
| H | **Ecosystem, community & go-to-market** | Publish to crates.io; write a "Laverna for X" tutorial series; target game-design + agentic-AI niches first. |

---

## 4. Dimension A — Architecture & module boundaries

### 4.1 Current state

The binary leaks 8 source-module paths (from `strings`): `src/verify/verifier.rs`, `src/descent/mod.rs`, `src/entity/mod.rs`, `src/main.rs`, `src/optimize/mod.rs`, `src/compute/parser.rs`, `src/compute/pipeline.rs`, `src/validation/math_gate.rs`. Plus inferred: `domain_graph/`, `astrology/`, `chart/`, `ephemeris/`, `formula/`, `scoring/`, `mcp/`, `router/`.

The architecture is **clean but monolithic** — everything is compiled into one 7 MB binary, including the ephemeris tables.

### 4.2 Target state — split into crates

```
laverna-core          ← the descent engine, NAND primitives, gyro router, wheel
laverna-tanto         ← the formula parser/evaluator (extractable as a crate)
laverna-corpus        ← versioned formulas + entities + nakshatras + rashis
laverna-ephemeris     ← thin wrapper that delegates to xalen-ephem (or vsop87)
laverna-optimize      ← the Pareto/MILP solver
laverna-chart         ← Vedic chart construction (depends on ephemeris + corpus)
laverna-mcp           ← the MCP server (depends on all of the above)
laverna-cli           ← the clap binary (depends on all of the above)
```

### 4.3 Concrete next steps

1. **Audit the ephemeris.** Run `chart` for a known date+location and compare lunar position against JPL Horizons.
2. **Extract `laverna-tanto` first.** It's the most reusable module. Publish to crates.io.
3. **Externalize the corpus as a TOML directory** with `corpus/version.toml` manifest.
4. **Document the descent cascade formally.**
5. **Introduce a `Profile` trait** — ship `VedicProfile` (default), stub `WuXingProfile`, `MBTIProfile`, `TemperamentProfile`.

---

## 5. Dimension B — Corpus & knowledge representation

### 5.1 Current state

- 528 formulas, each with: id, primary graha, inputs, output formula (Tanto expression), and implicit domain.
- 214 entities, each with: id, name, description, properties (key-value), tags.
- Embedded at compile time, no version number, no community-extension surface.

### 5.2 Target state

**A versioned, graph-structured, community-extensible corpus.** Add `corpus/version.toml` with semver + content hash. Promote formulas to **typed graph nodes** with `relations`, `source`, `confidence` fields. Allow users to drop `*.toml` files into `~/.laverna/corpus/` to extend the seed corpus.

### 5.3 Concrete next steps

1. **Extract the current corpus to TOML files** (`laverna corpus export`).
2. **Write `laverna corpus validate`** — checks Tanto parsability, graha references, no duplicate ids.
3. **Write `laverna corpus diff`** between versions.
4. **Add the `~/.laverna/corpus/` overlay loader.**
5. **Generate the corpus graph** as DOT/GraphML export.

---

## 6. Dimension C — Solver & optimization

### 6.1 Current state

- `optimize` solves point-allocation under budget + prerequisite + cost constraints.
- Algorithm: **exact enumeration of Pareto-optimal allocations**.
- Cycle detection in `requires` works. `--top-k N` returns N distinct Pareto-optimal solutions.

### 6.2 The scaling problem

Exact Pareto enumeration is **exponential in the number of items**. For 50 items with max_level 20 and a 100-point budget, it's intractable.

### 6.3 Target state

**Default: exact enumeration (current behavior). Optional: MILP backend via `good_lp` for scale.** Gated behind a feature flag.

```bash
laverna optimize --schema my.toml                      # exact, deterministic
laverna optimize --schema my.toml --solver milp        # MILP, fast, requires feature
laverna optimize --schema my.toml --pareto-sweep --steps 11
```

### 6.4 Concrete next steps

1. **Benchmark the current exact solver.** Find where it crosses 1 second.
2. **Add `good_lp` + HiGHS** as optional `--solver milp` backend.
3. **Implement `--pareto-sweep`** as a weighted-sum sweep.
4. **Add `--validate-only`** to `optimize`.
5. **Document the solver semantics precisely.**

---

## 7. Dimension D — MCP compliance & agentic surface

### 7.1 Current state

- MCP server runs on stdin/stdout via `laverna mcp`.
- Protocol version: **2024-11-05** (first public spec — 3 revisions behind).
- Latest stable: **2025-11-25**.
- Exposes 7 tools: `solve`, `entity_get`, `chart`, `validate`, `formulas`, `entities`, `optimize`.
- Missing from CLI: `route`, `build`, `websearch`, `ping`, `info`, `schema`.

### 7.2 What the spec gained in 3 revisions

| Version | Feature | Laverna status | Priority |
|---------|---------|----------------|----------|
| 2025-03-26 | Tool annotations (read-only / destructive / idempotent) | Present | — |
| 2025-03-26 | Streamable HTTP transport | Missing | Medium |
| 2025-06-18 | Structured tool output (`structuredContent`) | Present | — |
| 2025-06-18 | `title` field | Present | — |
| 2025-06-18 | Elicitation | Missing | Medium |
| 2025-06-18 | Resource Links | Missing | Low |
| 2025-11-25 | Icons | Missing | Low |
| 2025-11-25 | JSON Schema 2020-12 | Unclear | Medium |
| 2025-11-25 | Experimental Tasks | Missing | Low |

### 7.3 Target state

- Bump protocol version to **2025-11-25**.
- Expose `route` and `build` as MCP tools (done in v0.2.0).
- Convert `formulas`, `entities`, `schema` to **Resources**.
- Add **Prompts** for common workflows.
- Add **tool annotations** (already present on v0.1.0 tools).
- Use **elicitation** for "fail-loud" cases (philosophically compatible with "never hallucinate").

---

## 8. Dimension E — Web integration (World Bank + beyond)

### 8.1 Current state

- `websearch` queries the World Bank Indicators API.
- **Two confirmed bugs** (see §1.5 #5 and #6) — both fixed in v0.2.0.
- Only one backend (World Bank). No caching. No ISO-3166 gazetteer (root cause of bug #5).

### 8.2 The World Bank API (correct usage)

- **Endpoint:** `https://api.worldbank.org/v2/country/{country_code}/indicator/{indicator_code}?format=json&date=YYYY:YYYY&per_page=1000`
- **Country codes:** ISO-3166 alpha-2 (`US`, `CN`, `IN`, `JP`, `DE`, ...).
- **Date range:** `date=2018:2023` (colon delimiter, single request).
- **Response shape:** `[pagination_meta, [observations...]]` — the data array is index `[1]`.
- **`value` is a string** and may be `null` for missing years.

### 8.3 Target state

- Fix the two bugs (done in v0.2.0).
- Add an embedded ISO-3166 gazetteer (done in v0.2.0).
- Add caching in `~/.laverna/cache/websearch/`.
- Add FRED / IMF / OECD / Eurostat backends.
- Add `laverna websearch catalog [--source <s>]`.

---

## 9. Dimension F — UX, CLI ergonomics & docs

### 9.1 Current state

- CLI is clean, clap-4 idiom, `--help` is excellent.
- `--format json` is supported on most subcommands (expanded in v0.2.0).
- No `laverna repl`. No `laverna doctor`. No docs site.
- **Three cosmetic bugs** (see §1.5 #1–3) — #1 and #2 not reproducible on current source; #3 fixed in v0.2.0.

### 9.2 Target state

- Add `--format json` everywhere (done for info/entity-get/formulas/entities/route/chart/validate/solve in v0.2.0).
- Canonical JSON (RFC 8785 JCS) for byte-stable output.
- `laverna repl` — persistent REPL with `:help`, `:quit`, `:load`, pipelines.
- `laverna doctor` — health check.
- Docs site (mdbook).

---

## 10. Dimension G — Determinism, proof objects & verification

### 10.1 Current state

Laverna's central claim is "never guesses, fails loud." Enforced by:
- 7-layer descent: tokens that don't reach `Nand` (d6) are flagged.
- Numerical expressions verified against stated RHS (`2 + 2 = 5` → `✗ mismatch`).
- Tanto parser rejects unparseable expressions.
- No ML in core.

But: **there is no externally-auditable proof object.** A `solve` result says "100% resolution" — but you can't re-verify that claim without re-running `solve`.

### 10.2 The Lean / Rocq lesson

Lean and Rocq separate **proof search** (may be heuristic) from **proof checking** (fast, deterministic, re-runnable). A proof object is a *persisted artifact* that can be re-verified by a separate tool.

### 10.3 Target state (PARTIALLY DONE in v0.2.0)

```bash
laverna solve --query "2 + 3 = 5" --proof-out proof.json   # emit proof object
laverna verify proof.json                                  # re-verify (fast, no descent)
```

Implemented in v0.2.0: `--proof-out` on `solve` and the `verify` subcommand (see `build_proof_object` / `cmd_verify`).

### 10.4 Proof object schema (proposed / implemented subset)

```json
{
  "schema_version": "1.0.0",
  "query": "2 + 3 = 5",
  "computed_at": "<unix-secs>",
  "laverna_version": "0.2.0",
  "descent": {
    "resolution_score": 1.0,
    "average_depth": 6.0,
    "layer_counts": [0,0,0,0,0,0,3],
    "nand_completeness": 1.0,
    "tokens": [ { "text": "2", "settled_layer": "Nand", "depth": 6, "is_absolute": true, ... } ]
  },
  "tanto_evaluations": [ { "token": "2", "expression": "...", "value": 2.0 } ],
  "dominant_domains": ["Mangala", "Shukra", "Brihaspati", "Ketu", "Budha", "Chandra"]
}
```

### 10.5 Concrete next steps

1. **Define the proof object schema** (done — subset in v0.2.0).
2. **Add `--proof-out` to `solve`** (done in v0.2.0).
3. **Add `laverna verify <path>`** (done in v0.2.0).
4. **Document the determinism contract precisely.**
5. **Add `laverna corpus hash`** subcommand.

---

## 11. Dimension H — Ecosystem, community & go-to-market

### 11.1 Current state

- Repo: 0 stars, 0 forks, 5 open issues, created 2 days ago.
- License: MIT.
- Distribution: GitHub releases only (4 musl/gnu targets, SHA256 checksums).
- Not on crates.io.
- No Discord/Slack/Discourse.
- Name collision: the unrelated JS note-taking app "Laverna" exists.

### 11.2 Target state

- **Publish to crates.io** as `laverna-core`, `laverna-tanto`, `laverna-corpus`, `laverna-cli`.
- **Pick the niche: game design + agentic AI** (the CP2077 schema is a worked example).
- **Content strategy**: "Laverna for game designers", "Deterministic reasoning for agentic AI".
- **Community surface**: Discord, GitHub Discussions, `CONTRIBUTING.md`, good-first-issues.
- **Address the name collision** — recommend rename to **"Sheffer"** before v0.2.

### 11.3 Concrete next steps

1. Publish to crates.io.
2. Write the "Laverna for game designers" tutorial.
3. Write the "Deterministic reasoning for agentic AI" essay.
4. Launch a Discord.
5. Tag good-first-issues.
6. Decide on the rename question.

---

## 12. The release roadmap

### 12.1 v0.2.0 — Stabilize & ship (1 week) — **IN PROGRESS / PARTIALLY COMPLETE**

- [x] Fix the 8 bugs (§1.5) — #3, #4, #5, #6 fixed in code; #1, #2 not reproducible on current source
- [x] Bump MCP protocol version to 2025-11-25
- [x] Add `route` and `build` as MCP tools (now 9 tools total)
- [x] Add `--proof-out` to `solve` + `verify` subcommand
- [x] Expose `--format json` on info / entity-get / formulas / entities (plus pre-existing solve/route/chart/validate)
- [ ] Write a real README
- [ ] Tag v0.2.0, publish GH release

### 12.2 v0.3.0 — Corpus graph + community extension (4 weeks) — **COMPLETE**

- [x] Externalize corpus to TOML files in `formulas/` + `entities/` (embedded by `build.rs`).
- [x] Add `laverna corpus validate/diff/export/graph` (plus `lint`).
- [x] Add `~/.laverna/corpus/` overlay loader (same-id override; `./corpus` also).
- [x] Add `source`, `confidence`, `relations` fields to formula schema.
- [x] Add `Profile` trait; stub `WuXingProfile`, `TemperamentProfile`.

### 12.3 v0.4.0 — Solver scale + REPL (3 weeks)

- [ ] Add `good_lp` + HiGHS as optional `--solver milp` backend.
- [ ] Add `--pareto-sweep`.
- [ ] Add `--validate-only` to `optimize`.
- [ ] Ship `laverna repl` behind a feature flag.
- [ ] Add FRED backend to `websearch`.

### 12.4 v0.5.0 — Elicitation + tasks (2 weeks)

- [ ] Add elicitation for `solve`'s "0% resolution" failure mode.
- [ ] Add `--strict` flag to disable elicitation.
- [ ] Pilot MCP Tasks for long-running `optimize` jobs.

### 12.5 Ongoing — Content & community

- [ ] Publish "Laverna for game designers" tutorial.
- [ ] Publish "Deterministic reasoning for agentic AI" essay.
- [ ] Launch Discord; recruit moderators.
- [ ] Write CONTRIBUTING.md.
- [ ] Scaffold mdbook docs site.
- [ ] Decide on rename question (recommend: rename to "Sheffer" before v0.2).

**Total timeline to v0.5.0: ~13 weeks (one quarter).**

---

## 13. The single most important thing

**Laverna's defensible differentiator is the proof object, not the astrology.**

The Vedic wheel is beautiful, original, and memorable — but it's a *classifier profile*, not the product. Every elevation move should pour concrete on:
1. The **7-layer NAND-to-bankai descent cascade**.
2. The **"never guesses, fails loud" contract**.
3. The **proof object** (once shipped).
4. The **embedded formula corpus + Pareto optimizer**.

If Laverna becomes the **"deterministic reasoning engine with verifiable proof objects"** that agentic AI developers reach for when they can't trust an LLM, it will have a category. If it tries to be "the Vedic astrology CLI with NAND proofs," it will be a footnote.

---

## Appendix A — The 7-layer descent cascade (inferred specification)

| Layer | Name | Depth | Behavior (inferred) |
|-------|------|-------|---------------------|
| d0 | Macro | 0 | Token recognized but no classification. |
| d1 | Micro | 1 | Token classified into ≥1 graha domain but no formula matched. |
| d2 | Mesa | 2 | Domain bound, but unification with entity failed. |
| d3 | Element | 3 | Domain + formula matched, entity bound, but not reduced to NAND. |
| d4 | Compound | 4 | Formula evaluated but result not yet provable down to NAND. |
| d5 | Nand | 5 | (internal-only) |
| d6 | Nand (user-facing) / Sheffer (internal) | 6 | Full reduction: domain + formula + entity + NAND-absolute classification. |

## Appendix B — The 16 lagna archetypes (observed)

`Stride, Cut, Threshold, Trail, Renewal, Bread, Crown, Repose, Vow, Gem, Target, Altar, Elder, Spear, Foundation, Shield, Acropolis, Ear, Rhythm, Remedy, Blaze, Depths, Guide`

## Appendix C — Key URLs

- Laverna: https://github.com/nutypebuddha/Laverna
- Vedākṣha: https://github.com/arthiqlabs/vedaksha
- XALEN: https://github.com/vedika-io/xalen-ephemeris
- MCP spec: https://modelcontextprotocol.io/specification
- VSOP87: https://docs.rs/vsop87
- World Bank API: https://datahelpdesk.worldbank.org/knowledgebase/articles/889392
- good_lp: https://crates.io/crates/good_lp
- NAND logic: https://en.wikipedia.org/wiki/NAND_logic

## Appendix D — Research methodology & limitations

Compiled from black-box CLI testing, binary strings extraction, and live web research. Limitations: GitHub API rate-limited; source code read only via binary + README; Moon-accuracy concern inferred from README; competitor feature counts from public READMEs.
