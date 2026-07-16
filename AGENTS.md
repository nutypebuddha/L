# L.ai · Proof — Agent Instructions (*Laverna* code name)

Vedic reasoning engine reboot. 4-layer architecture (functional module names):
**aspect** → **nlp/query** → **verify** (over **primitive** NAND gates at the base).
NAND gate primitives at the bottom. Determinism-first.

Part of the **L.ai** umbrella (see `docs/brand.md`): *Laverna* = **L.ai · Proof**
(deterministic proof function). Sister L.ai functions: **L.ai · Gate** (CID,
per-token validation) and **L.ai · Bridge** (CID-Bridge, chatbot fan-out).

## Environment
- aarch64 Linux, proot-distro Debian container on Android/Termux
- Workspace: `/root/Laverna/`
- check disk before building: `df -h / | tail -1`
- `CARGO_BUILD_JOBS` is NOT hardcoded — set per-invocation
- `/sdcard` is bind-mounted from `/storage/emulated/0` (Android internal storage)
- proot path: `/data/data/com.termux/files/usr/var/lib/proot-distro/containers/debian/rootfs`
- **Shared hub**: `/sdcard/Download/Laverna/` — the one folder shared with the
  Android "My Files" app (scoped storage forbids new top-level dirs, so it nests
  under `Download/`). `bin/` = export target (x86_64 only); `tickets/` = intake.

## Dev cycle
```bash
cargo clippy -- -D warnings && cargo test --lib && cargo fmt -- --check
```

## Ticket intake
Tickets are Markdown files in **Termux's private downloads** — `~/downloads`
(`/data/data/com.termux/files/home/downloads/`). proot reads this dir *directly*,
so there is **no Android per-app FUSE cache gap** (shared `/sdcard` has one:
files dropped there from another app such as "My Files" may not appear to proot
until that FUSE mount's dir cache expires — `scripts/tickets.sh --refresh` nudges
it, but the Termux-private dir avoids the problem entirely).
```bash
scripts/tickets.sh                       # list ticket files (default: ~/downloads)
TICKETS_DIR=/some/dir scripts/tickets.sh # override source
```
When the user says "scan tickets", list `~/downloads/*.md` and read the newest /
relevant one. Each file's `# Title` is the H1; a file may hold one or several
tickets (e.g. `laverna-0.3.0-tickets.md` holds T52–T57). The strategic roadmap
lives in `docs/ELEVATING_LAVERNA.md`; these files are the actionable work queue.
Built binaries still export to the shared hub `/sdcard/Download/Laverna/bin/` so
they're reachable from the Android "My Files" app.

## CI gate order
`cargo fmt -- --check` → `cargo clippy --features "graph,milp,llm" -- -D warnings` →
`cargo test --lib --features "graph,milp"` → `cargo test --features llm --lib`

## Build & features
```bash
# Full-features x86_64 release build + export to the shared hub.
# Wraps the cargo build and copies to /sdcard/Download/Laverna/bin/laverna-x86_64.
scripts/export.sh

# Equivalent manual invocation:
cargo build --release --target x86_64-unknown-linux-gnu --features "mcp websearch budget llm milp graph"
cp target/x86_64-unknown-linux-gnu/release/laverna /sdcard/Download/Laverna/bin/laverna-x86_64

# Native build (for testing on host)
cargo build --release --features "graph,milp"
```
| feature | enables | default |
|---------|---------|---------|
| `mcp` | ureq web search integration | no |
| `websearch` | ureq (World Bank stats) | via `mcp` |
| `budget` | token budget tracking | no |
| `bench` | criterion harness | no |
| `llm` | marker for stub inference module (no heavy deps) | no |
| `milp` | MILP solver via good_lp/microlp (T46) | no |
| `graph` | Graph algorithms via petgraph (T47) | no |

Dead deps removed (P0): `sha2`, `postcard`, `rmcp`, `tokio`, `llama-gguf`.

## Elevation roadmap

`docs/ELEVATING_LAVERNA.md` is the strategic plan (black-box audit + competitive
research). Thesis: Laverna's differentiator is the **NAND-to-verify proof cascade**
+ **fail-loud contract** + **deterministic optimization** — NOT the Vedic astrology
layer (Vedākṣha/XALEN out-feature it there). Astrology is a *classifier profile*,
not the product.

### v0.2.0 status (completed)
- `chart` lagna now prints symbol `♏` (was duplicated name) — `main.rs:1597`
- `route` warns when both `--query` and `--repos` given (was silent ignore)
- `websearch`: ISO-3166 gazetteer + indicator lookup table (fixes compound-query
  mis-segmentation); 10s HTTP timeout + `per_page=1000` + `date=2018:2023` range
  (fixes multi-year hang). See `parse_websearch_query` / `world_bank_lookup`.
- MCP protocol bumped `2024-11-05` → `2025-11-25`; `route` + `build` tools added
  (9 tools total).
- `solve --proof-out <path>` emits a proof object; `laverna verify <path>`
  re-checks it (`build_proof_object` / `cmd_verify`).
- `--format json` added to `info`, `entity-get`, `formulas`, `entities`.

### v0.3.0 status (completed)
- `corpus` subcommand: `export`, `validate`, `diff`, `graph`, `lint` — all load
  from the embedded corpus (or a `<dir>` of `formulas.toml`/`entities.toml`).
  - `validate`: **structural** Tanto parse check (tolerates division-by-zero
    special cases, unlike the old `verify_expression` NL-proposal verifier which
    false-positived on every formula). Skips `llm`-type formulas (prompt templates).
  - `lint`: flags expressions referencing undeclared variables (not in `inputs`,
    known constants, or other formula ids) and formulas with no graha/zodiac tag.
    Advisory (exit 0). Caught 4 real corpus bugs (missing `inputs`, a `restructing`
    typo).
  - `graph`: DOT of formula input→output dependencies.
- **Overlay loader** (`~/.laverna/corpus/`, `./corpus`): user `*.toml` files merge
  over the embedded seed corpus — same-id formulas override. Surfaced as the
  `corpus-overlay` feature in `info`.
- **Tanto** gained well-defined builtins: `erf`, `factorial`, `gcd`, `gauss_inv`
  (probit, Acklam), `diff`, `log` (natural + 2-arg base). Added `parse_math`
  (structural parse) and `is_expression_valid`.
- Formula schema extended with `source`, `confidence`, `relations` (serde-defaulted,
  backward compatible) for richer corpus/graph metadata.
- `Profile` trait + stubs `WuXingProfile` (Five Elements) and `TemperamentProfile`
  (four humors) — alternative classifier lenses over the corpus, per the
  "astrology is a *profile*, not the product" thesis. `src/profile.rs`.
- **Versioned corpus**: `build.rs` emits `CORPUS_VERSION` + `CORPUS_CONTENT_HASH`
  (FNV-1a over the embedded corpus — no crypto deps) + `CORPUS_VERSION_TOML`.
  `corpus hash` prints them; `corpus export` writes `version.toml`. Detects
  corpus drift across builds / community forks.
- `README.md` written (real usage doc).

The seed corpus (formulas, synonyms, nonmath, shikai forms, events, entities) is
**always embedded** in the binary by `build.rs` (no feature gate) — `entities` /
`formulas` / `entity-get` load from any CWD, not just the repo root. `info`
reports `embedded-corpus` to make this explicit.

## Architecture
- **Layer 0 — Primitive**: `src/primitive/`, `src/descent/`, `src/router/`
- **Layer 1 — Aspect**: `src/aspect/`, `src/formula/`, `src/entity/`, `src/ephemeris/`, `src/chart/`
- **Layer 2 — NLP/Query**: `src/nlp/`, `src/query/`
- **Layer 3 — Verify**: `src/verify/`, `src/mcp/`
- **Cross-cutting**: `src/optimize/`, `src/build/`, `src/graph/` (T47), `src/hungarian/` (T48), `src/csp/` (T49)

Pipeline: query → nlp_parse → descent_engine → query_process → verify_solve

### `laverna build` (T45)
Chains chart → graha weight mapping → optimize in one command.
Domain profiles live in `domains/*.toml`. Weight computation:
`objective.weights[score] = Σ (pillar_weight[graha] × split[fraction])`.
Shares the solver with `optimize` — no subprocess spawning.

### Optimization Primitives (T46-T49)
Canonical OR primitives as composable building blocks.
- **T46**: State-space guard (NODE_CAP=5M) + MILP tier via `good_lp`/`microlp` (`milp` feature)
- **T47**: Graph primitives via `petgraph` (`graph` feature): Dijkstra, Kruskal, max-flow
  - **CRITICAL**: petgraph's HashMap-ordering → sort by stable key before output
- **T48**: Hand-rolled Hungarian algorithm for assignment problems (`src/hungarian/`)
- **T49**: Hand-rolled CSP backtracking solver with fixed variable/value ordering (`src/csp/`)

Schema `shape` field: `"knapsack"` (default) | `"milp"` | `"assignment"` | `"shortest_path"` | `"mst"` | `"max_flow"` | `"interval_scheduling"` | `"csp"`

### Build subcommand flags (T51)
`-d` / `--domain` — path to TOML profile. `-t` / `--datetime` — ISO datetime.
Short flags must never collide within a subcommand.

## Performance Optimizations
- **Entity cache**: `generate_dynamic_entity` computed once per token, cached in `DescentEngine` via `HashMap<String, DynamicEntity>` passed through descent pipeline
- **In-place VedicClassification**: `set_graha()`, `set_sign()`, `merge_max_into()` — `&mut self` methods eliminate clone-and-reassign patterns
- **Single Dijkstra**: `reconstruct_path` uses pre-computed distances instead of re-running Dijkstra
- **Pareto frontier**: single-objective fast path + sorted multi-objective with early termination
- **Stopwords**: `LazyLock<HashSet>` for O(1) lookup instead of O(n) slice scan
- **Formula load optimization**: deferred word_index via `finalize()` + buffer-reuse `write_searchable_text()` — eliminates ~3,500 `format!` + ~1,200 `String::new()` per batch
- **evaluate_expr fast path**: skips `preprocess_line` alloc roundtrip for pure-math expressions (digit/operator/paren start, no " of ")

## Determinism Rule (T43, T50 — applies everywhere)
**Every** output path that touches a `HashMap` (or any unordered collection) must
sort by a stable key before printing/aggregating. This includes:
- `--explain` trace output (T50 fix)
- `scoring.*` aggregation steps
- petgraph algorithm results (T47 — dijkstra returns `HashMap<NodeId, K>`)
- `domain_graph/graph.rs` `neighbors()` output (sorted by domain index)
- Any future `BTreeMap`-free code path

Treat unsorted iteration over HashMap as a **correctness bug**, not a style issue.

## Naming Conventions (GNU/UNIX Pure Function Style)

All functions must be **pure**: no side effects, deterministic, all inputs as
parameters, all outputs as return values. No global state.

### Functions
- `snake_case` — always
- **Verb-first**: `compute_*`, `evaluate_*`, `validate_*`, `transform_*`, `encode_*`, `decode_*`
- **Module prefix** when disambiguation needed: `nand_gate()`, `nand_not()`, `nand_and()`
- **No abbreviations**: `accumulator` not `acc`, `left_operand` not `lhs`
- **Bool predicates**: `is_*`, `has_*`, `can_*`

### Types
- `PascalCase` — always
- **Suffix by role**: `*Registry`, `*Engine`, `*Gate`, `*Result`, `*Config`

### Constants
- `SCREAMING_SNAKE_CASE`

### Modules
- `snake_case` — single word preferred

### Example
```rust
/// Pure function: NAND gate. Universal gate — all others derive from this.
pub fn nand_gate(left_input: bool, right_input: bool) -> bool {
    !(left_input && right_input)
}

/// Pure function: Half adder. Returns (sum, carry).
pub fn half_adder(left_operand: bool, right_operand: bool) -> (bool, bool) {
    let sum = xor_gate(nand_gate(left_operand, right_operand), or_gate(left_operand, right_operand));
    let carry = and_gate(left_operand, right_operand);
    (sum, carry)
}
```

## License (Apache-2.0)

Laverna is licensed under **Apache-2.0** (sole author, no outside contributors —
relicensed from MIT). `Cargo.toml` `license` field, `LICENSE`, and `NOTICE`
must stay in agreement. New source files should open with:

```rust
// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0
```

Attribution lives in `NOTICE`; per-file headers are advisory.

## Conventions
- Formulas, not facts: encode relationships, not static lookups
- Cross-domain by default: new formulas reference ≥2 grahas
- Commits: Conventional Commits (`feat(domain_graph):`, `fix(verify):`)
- Errors: `anyhow` at call sites, `thiserror` for library types
