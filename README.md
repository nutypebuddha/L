# Ł.AI · Proof — *Laverna*

**Deterministic verification engine** — a 9-graha (Navagraha) wheel, a
NAND-to-bankai compute cascade, and verifiable proof objects. Rust, edition 2021.

*Laverna* is the code name for **Ł.AI · Proof**: the deterministic-proof pure
function in the Ł.AI substrate (see [`docs/brand.md`](docs/brand.md)). The other
Ł.AI functions are **Ł.AI · Gate** (CID, per-token validation) and
**Ł.AI · Bridge** (CID-Bridge, chatbot fan-out).

Ł.AI · Proof never guesses and fails loud: every result either carries a
derivation back to primitive NAND gates or is explicitly marked unproven
(e.g. LLM-estimated steps, confidence-penalized). No hallucination, no silent
fallback.

> **Thesis.** The differentiator is the **NAND-to-bankai proof cascade** plus the
> **fail-loud, no-hallucination contract** plus **deterministic optimization** —
> not the Vedic chart. The astrology layer is a *classifier profile*, not the product.

## Architecture

Four layers, deterministically composed:

```
Layer 3 — Bankai       Expression verification, diagnostics, confidence scoring
Layer 2 — Zanpakutō    NLP tokenization, intent parsing, domain classification
Layer 1 — Asauchi      Formula registry, entity registry, ephemeris, charts
Layer 0 — Primitive    NAND gates, descent engine, gyro router (9-graha wheel)
```

Pipeline: `query → zanpakuto_nlp → descent_engine → shikai_process → bankai_solve`.

The embedded corpus (528 formulas, 214 entities) is compiled into the binary by
`build.rs`, so it works from any working directory.

## Build

```bash
# Default (minimal) build
cargo build --release

# Full-features build (all capabilities)
cargo build --release --features "mcp websearch budget llm milp graph"

# Run the test suite + lints (CI gate)
cargo fmt -- --check
cargo clippy --features "graph,milp,llm" -- -D warnings
cargo test --lib --features "graph,milp"
```

Exports are produced as `x86_64-unknown-linux-gnu` binaries (the target platform
for Termux). `cargo build --target x86_64-unknown-linux-gnu ...` then copy the
binary to an executable location.

## Subcommands

| Command | Purpose |
|---------|---------|
| `info` | Build/feature/corpus summary |
| `solve` | Resolve a query through the descent cascade; `--proof-out <path>` writes a proof object |
| `verify <path>` | Re-check a proof object produced by `solve --proof-out` |
| `formulas` / `entities` / `entity-get` | Inspect the corpus (`--format json` supported) |
| `chart` | Vedic chart from a datetime |
| `route` | Route a query across repositories / the corpus |
| `build` | Chain chart → graha weighting → optimization |
| `optimize` | Deterministic 7-pillar / Pareto solver (knapsack, MILP, assignment, shortest-path, MST, max-flow, CSP) |
| `websearch` | World Bank statistics lookup (ISO-3166 gazetteer) |
| `corpus` | Corpus tooling — see below |
| `ping` | Liveness check |

### `laverna corpus`

The corpus is externalized as TOML so it can be versioned and community-extended.

```bash
laverna corpus export <dir>     # write formulas.toml, entities.toml, version.toml
laverna corpus validate [dir]   # structural Tanto parsability + graha refs + dup ids
laverna corpus lint [dir]       # deeper: undeclared variables, missing graha tags
laverna corpus diff <a> <b>     # JSON diff of two exported corpora
laverna corpus graph [dir]      # DOT graph of formula input→output dependencies
laverna corpus hash             # embedded corpus semver + content hash
```

**Community extension.** Drop `*.toml` files (with `[[formula]]` / `[[entity]]`
tables) into `~/.laverna/corpus/` or `./corpus/`. They merge over the embedded
seed corpus; a same-id entry **overrides** the seed. `info` reports
`corpus-overlay` when an overlay is active.

### Proof objects & MCP

`laverna solve --proof-out proof.json` emits a machine-checkable proof object;
`laverna verify proof.json` re-derives and checks it. The same surface is exposed
over MCP (protocol `2025-11-25`) with 9 tools, including `route` and `build`.
This is the **Ł.AI · Proof** MCP surface; **Ł.AI · Gate** (CID) exposes 13 tools
and **Ł.AI · Bridge** fans both out to chatbots.

## Determinism

Outputs that touch unordered collections (HashMap / petgraph results) are sorted
by a stable key before printing or aggregating. The same inputs always yield the
same outputs and the same proof object.

## Brand

Part of the **Ł.AI** umbrella (offline, WASM-native, fail-loud verification
substrate). *Laverna* is the code name for **Ł.AI · Proof**. See
[`docs/brand.md`](docs/brand.md). Commercial mark: **Ł.AI** (DBA "Wintermore
Housekeeping"); "Laverna" is not filed as a trademark.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and
[NOTICE](NOTICE). Copyright 2026 nutypebuddha.
