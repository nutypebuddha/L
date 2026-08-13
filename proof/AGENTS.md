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
- **Epistemic provenance tags (Item 4).** `chart` JSON tags the fully-computed
  `chart` block `epistemic: {kind:"computed", source:"vsop87+lahiri",
  verified_against:"swiss_oracle"}` and the fully-modeled `personality` block
  `epistemic: {kind:"modeled", source:"bphs_shadbala", coefficients:[...]}`.
  Present `computed` values (planet longitudes, lagna, bhavas, aspects) with
  confidence; never present a `modeled` value (archetype, pillar weights,
  `dominant`) in the same voice. T79/T80: modeled values looked identical to
  computed ones for a whole release — the tag makes the boundary structural.
- **`gate validate` verdict is a stable enum, not a bool (Item 3).** The JSON
  carries `verdict` (`ok` | `corrected` | `failed` | `unevaluable`) and
  `fix_count`; the old `passed: bool` is gone. Branch on `verdict` — a
  `corrected` claim is one L silently fixed (do not report it as passing),
  `failed` is wrong-and-unfixable, `unevaluable` means L had nothing to judge
  against. A `corrected`/`failed` claim is never `ok`.
- **Refusals are a stable enum, not prose (Item 1).** `solve` JSON carries
  `"refusals": [{"refused": {"kind": "...", ...}}]`. Branch on `kind`
  (`ambiguous` | `no_name_anchor` | `unit_mismatch` | `search_too_large`) — the
  `detail` string may change but `kind` is API surface. Recommended responses:
  `ambiguous` → ask the person which reading they meant; `no_name_anchor` →
  rephrase naming inputs, retry once; `unit_mismatch` → tell the person their
   units don't compose, do **not** retry; `search_too_large` → split the query,
   retryable. The `[bind] REFUSED: …` stderr line is preserved for humans but is
   not a contract.
- **Binder trace is a structured document, not a story (Item 2).** `solve
   --explain-binding=json` attaches `explain_binding` to the solve JSON:
   `{formula_chain:[String], candidates_considered, tie_count, inputs:[{input,
   formula_id, hop, anchored_by, distance, method, pre_conversion_value,
   pre_unit, post_unit, converted}]}`. `formula_chain` is the *ordered* list of
   formula ids that actually produced the answer (last entry = formula that
   ultimately bound); for a single-rule query it has one element, a two-hop chain
   has two (T102). Each input also carries `formula_id` + `hop` (T101) so a chain
   never yields two unattributable entries with the same input name — `hop:0` is
   the top formula, `hop:1` the downstream rule. Use this to answer "why did it
   read N as input X" — `method` is `name`/`unit`/`derived`, `anchored_by` is the
   *actual* anchor token the binder selected (not a re-derivation), `distance` is
   the token distance (or `null` for a `derived` fill), `tie_count > 1` means the
   binding was effectively arbitrary (arbitrary → treat the binding as low-trust).
   Every input the formula consumed is recorded — including inputs filled from a
   prior hop's *derived* output (`method:"derived"`, `distance:null`, carrying the
   upstream value+unit) — so a two-hop chain never silently shows fewer inputs than
   it has (T100). The trace only records the *chosen* chain, not every candidate
   that bound during ranking (ranking uses a throwaway trace; the chosen formula is
   re-bound into the real trace, T101/T102). The `=text` mode (or no value) prints
   the human prose to stderr and omits the field from JSON, so `-f json` stdout
   stays clean.
- **Determinism receipt (Item 5).** Every proof JSON (`solve -f json` and
   `solve --proof-out`) carries `determinism_receipt`:
   `{laverna_version, features:[…], corpus_version, corpus_content_hash,
   effective_corpus_hash, query_hash}`. `features` is single-sourced from
   `collect_features()` (T99) — cargo flags plus `embedded-corpus`, and
   `corpus-overlay` when an overlay directory is present at runtime.
   `corpus_content_hash` is the compile-time embedded-corpus hash;
   `effective_corpus_hash` folds any runtime overlay TOML contents into the seed
   hash, so two runs over *different* corpora get different receipts (T99 — a
   receipt that under-reports is worse than none).    `query_hash` pins the normalized
   query ("this question") — normalized as lowercase + plural-strip + whitespace
   collapse, matching the binder's own token rule — so the receipt stands alone as
   a reproduce instruction even though the binder also sees the raw query.
   Before trusting two proofs as comparable, compare their receipts — a different
   `effective_corpus_hash`, `features`, or `query_hash` means the runs reasoned
   over a different build/KB/question even if the rest of the proof matches. The
   receipt is part of the hashed payload, so it is also covered by `digest`.
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
