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

See root `AGENTS.md` for the full table. `proof` (laverna) default already
enables `["assistant","mcp","websearch","budget","milp","graph"]`; a plain
`cargo build` pulls heavy deps. Use `--no-default-features` for a minimal build.
`llm` and `assistant-web` are the notable opt-in flags.

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

- 538 formulas, 222 entities — **embedded at compile time** by `build.rs`.
  `lai info` is the authoritative live count; update this line if it drifts.
- Overlay: `~/.laverna/corpus/` or `./corpus/` — user TOML merges over seed.
- After editing formula TOML: rebuild required (build.rs re-embeds).
- `corpus lint` catches undeclared variables and missing graha tags (advisory).

## AI / machine interface

Lai is built to be driven by an agent, not just read by a human. Prefer the
JSON surface over scraping `--help`/text.

- **Bootstrap with `lai info --format json`.** One call returns `version`,
  `features`, `formulas`, `entities`, `entity_domains`, and `subcommands`
  (name + one-line description for every subcommand). Discover capabilities
  from this manifest instead of parsing help text.
- **`--format json`** is supported by the analysis subcommands (`solve`,
  `route`, `chart`, `validate`, `optimize`, `build`, `strategize`, `verify`,
  `formulas`, `entities`, `corpus`, `gate`, …). Parse the JSON; never regex
  the text form.
- **Fail-loud is in the JSON.** `validate`/`solve` emit `passed`,
  `error_count`, `refusals`, `diagnostics`. Out-of-scope input is *refused*,
  never silently accepted — trust `passed:false`/`refusals`, not absence of
  output.
- **Refusals are a stable enum, not prose (Item 1).** `solve` JSON carries
  `"refusals": [{"refused": {"kind": "...", ...}}]`. Branch on `kind`
  (`ambiguous` | `no_name_anchor` | `unit_mismatch` | `search_too_large`) — the
  `detail` string may change but `kind` is API surface. Recommended responses:
  `ambiguous` → ask the person which reading they meant; `no_name_anchor` →
  rephrase naming inputs, retry once; `unit_mismatch` → tell the person their
  units don't compose, do **not** retry; `search_too_large` → split the query,
  retryable. The `[bind] REFUSED: …` stderr line is preserved for humans but is
  not a contract.
- **Deterministic.** Outputs are sorted by stable key (determinism rule in
  root AGENTS.md); identical input → byte-identical JSON across runs. Safe to
  diff or cache.
- **Native integration: `lai mcp`** speaks JSON-RPC over stdio (the MCP
  server) for interactive multi-turn sessions; CLI + `--format json` for
  one-shot calls.
- **`lai schema <subcommand>`** prints the canonical TOML input template
  (self-describing input shapes).

## Known gotchas

- **`DOMAIN_PROFILE_TEMPLATE`** (`build/mod.rs`): defines `score_cool` in graha_map
  — template is now complete (T53 fix). If you see `unknown score 'score_cool'`,
  the template is stale.
- **`strategize --budget >20`**: uses LP-relaxation branch-and-bound (T52 fix).
  Budget=30 completes in ~0.6s. Test: `branch_and_bound_handles_large_budget`.
- **petgraph results** must be sorted by stable key before output (determinism rule).
- **`build.rs`** embeds corpus + version hash — two sequential builds differ.
