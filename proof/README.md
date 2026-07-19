<div align="center">

# Ł L.ai · *Proof*

**Offline, deterministic verification for AI. *Verify, don't trust.***

> Part of the **[L.ai](https://github.com/nutypebuddha/lai)** ecosystem.

<img src="../docs/brand-LAI.svg" width="72" height="96" alt="L.ai witch-hat Ł glyph"/>

</div>

---

**L.ai · Proof** is an offline, deterministic verification engine. It answers factual,
numeric, and computable questions by running them through a NAND-to-verify
compute cascade and an embedded, content-addressed corpus — then hands back a
machine-checkable **proof object** or a typed **refusal**. No guessing, no
hallucination, no network required at runtime.

*Laverna* is the internal code name for **L.ai · Proof** (see
[`docs/BRAND.md`](docs/BRAND.md)). The sibling L.ai functions are **L.ai · Gate**
(CID, per-token validation) and **L.ai · Bridge** (CID-Bridge, chatbot fan-out).

L.ai · Proof never guesses and fails loud: every result either carries a
derivation back to primitive NAND gates or is explicitly marked unproven. The
differentiator is the **NAND-to-verify proof cascade** plus the **fail-loud,
no-hallucination contract** plus **deterministic optimization** — not the Vedic
chart. The astrology layer is a *classifier profile*, not the product.

---

## 5-minute offline assistant

You get a plain-talking assistant that checks every factual claim before it
answers — all on-device.

```bash
# 1. Build with the local-LLM + MCP features
cargo build --release --features "mcp websearch budget llm milp graph"

# 2. Fetch a small model (one command; no API key, no auth)
./scripts/get-model.sh

# 3. Talk to it over MCP (any MCP client, or the bundled stdio server)
./target/release/laverna mcp
```

No model? The assistant still answers from the verified engine and tells you
it did — it never fabricates. Run `./scripts/get-model.sh` to give it a voice.

Prebuilt static binaries + the bundled llama.cpp engine are attached to each
GitHub release (see the `laverna-x86_64-bundle.tar.gz` asset).

---

## Architecture

Four layers, deterministically composed (module names are functional):

```
Layer 3 — verify       Expression verification, diagnostics, confidence scoring
Layer 2 — nlp/query    NLP tokenization, intent parsing, domain classification
Layer 1 — aspect       Formula registry, entity registry, ephemeris, charts
Layer 0 — primitive    NAND gates, descent engine, router (9-graha wheel)
```

Pipeline: `query → nlp_parse → descent_engine → query_process → verify_solve`.

The embedded corpus (528 formulas, 214 entities) is compiled into the binary by
`build.rs`, so it works from any working directory.

## Build & test

```bash
# Default (minimal) build
cargo build --release

# Full-features build (all capabilities)
cargo build --release --features "mcp websearch budget llm milp graph"

# CI gate
cargo fmt -- --check
cargo clippy --features "graph,milp,llm" -- -D warnings
cargo test --lib --features "graph,milp"
```

### WebAssembly (L.ai · Proof in the browser)

`laverna-wasm/` compiles the deterministic `solve` / `evaluate` / `verify` /
`sha256` functions to a ~590KB WASM module.

```bash
cd laverna-wasm
bash build.sh                 # -> www/ (JS + .wasm bindings)
cd www && python3 -m http.server 8080
# open http://localhost:8080
```

The browser demo calls the pure functions directly — no server, fully offline.
`laverna-wasm` exposes: `init`, `evaluate(expr)`, `solve(schema_json)`,
`verify(proof_json)`, `sha256(input)`.

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
over MCP (protocol `2025-11-25`) with 10 tools, including `route`, `build`, and
`laverna_companion` (the plain-language assistant). This is the **L.ai · Proof**
MCP surface; **L.ai · Gate** (CID) exposes 13 tools and **L.ai · Bridge** fans
both out to chatbots. See [`docs/mcp-registry.md`](docs/mcp-registry.md) to list
L.ai in an MCP registry.

## Determinism

Outputs that touch unordered collections (HashMap / petgraph results) are sorted
by a stable key before printing or aggregating. The same inputs always yield the
same outputs and the same proof object.

## Brand

Part of the **L.ai** umbrella (offline, WASM-native, fail-loud verification
substrate). *Laverna* is the code name for **L.ai · Proof**. See
[`docs/BRAND.md`](docs/BRAND.md). Commercial mark: **L.ai** (DBA "Wintermore
Housekeeping"); "Laverna" is not filed as a trademark.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) and
[NOTICE](NOTICE). Copyright 2026 nutypebuddha.
