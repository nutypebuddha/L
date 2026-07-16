# L.ai · Proof — Agent Instructions

Vedic reasoning engine. 4-layer architecture: **aspect** → **nlp/query** →
**verify** (over **primitive** NAND gates). Part of the L.ai umbrella — see
root `AGENTS.md` and `docs/brand.md`.

## Quick commands

```bash
# CI gate (proof-only, feature-gated)
cargo fmt -- --check
cargo clippy -p laverna --features "graph,milp,llm" -- -D warnings
cargo test -p laverna --lib --features "graph,milp"
cargo test -p laverna --lib --features llm

# single test
cargo test -p laverna --lib optimize::tests::branch_and_bound_handles_large_budget

# full-features build (native)
cargo build --release -p laverna --features "mcp websearch budget llm milp graph"
```

## Architecture

```
Layer 0 — Primitive   src/primitive/, src/descent/, src/router/
Layer 1 — Aspect      src/aspect/, src/formula/, src/entity/, src/ephemeris/, src/chart/
Layer 2 — NLP/Query   src/nlp/, src/query/
Layer 3 — Verify      src/verify/, src/mcp/
Cross-cutting         src/optimize/, src/build/, src/graph/, src/hungarian/, src/csp/
```

Pipeline: `query → nlp_parse → descent_engine → query_process → verify_solve`

## Feature flags

See root `AGENTS.md` for the full table. Key gotcha: `default = []` — minimal
build has no heavy deps. MCP requires `--features mcp`. MILP requires `--features milp`.

## Schema shapes

`optimize` schema `shape` field selects the solver:
`"knapsack"` (default) | `"milp"` | `"assignment"` | `"shortest_path"` | `"mst"` | `"max_flow"` | `"interval_scheduling"` | `"csp"`

## `laverna build`

Chains chart → graha weight mapping → optimize. Domain profiles in `domains/*.toml`.
Weight formula: `objective.weights[score] = Σ (pillar_weight[graha] × split[fraction])`.
Shares solver with `optimize` — no subprocess.

## Export

```bash
proof/scripts/export.sh   # static x86_64-musl build → /sdcard/Download/Laverna/bin/
```

Note: `REPO_ROOT` in export.sh resolves to `proof/` but target dir is workspace root.
`/sdcard` is vfat FUSE — no symlinks, no exec bits.

## Corpus

- 528 formulas, 214 entities — **embedded at compile time** by `build.rs`.
- Overlay: `~/.laverna/corpus/` or `./corpus/` — user TOML merges over seed.
- After editing formula TOML: rebuild required (build.rs re-embeds).
- `corpus lint` catches undeclared variables and missing graha tags (advisory).

## Known gotchas

- **`DOMAIN_PROFILE_TEMPLATE`** (`build/mod.rs`): defines `score_cool` in graha_map
  — template is now complete (T53 fix). If you see `unknown score 'score_cool'`,
  the template is stale.
- **`strategize --budget >20`**: uses LP-relaxation branch-and-bound (T52 fix).
  Budget=30 completes in ~0.6s. Test: `branch_and_bound_handles_large_budget`.
- **petgraph results** must be sorted by stable key before output (determinism rule).
- **`build.rs`** embeds corpus + version hash — two sequential builds differ.
