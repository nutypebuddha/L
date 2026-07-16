# L.ai

> **Verify, don't trust.**

**L.ai** is a single, offline-first verification umbrella for AI. One project,
four functions — all deterministic, all fail-loud, none requiring a network at
runtime:

| Function | Crate / dir | What it does |
|----------|-------------|--------------|
| **L.ai · Proof** | `proof/` (`laverna`) → `lai` | Deterministic reasoning engine: 9-graha wheel, NAND-to-verify proof cascade, embedded corpus, machine-checkable proof objects, local LLM assistant. |
| **L.ai · Gate** | `gate/` (`lai-gate`) → `lai gate` | Per-token validation layer for LLM output — math, logic, fact, fallacy, bias. ~630KB pure Rust + WASM. |
| **L.ai · Bridge** | `bridge/` (Node/TS) | Universal MCP bridge — any chatbot (Grok, Claude, GPT, Mistral) hooks into Gate validation through one endpoint. |
| **L.ai · Athena** | `athena/` (`athena`) → `lai athena` | Relational reasoning engine — 30 subcommands for formulas, entities, Shikai NLP, Bankai solve, zodiac wheel, gyro descent. |

This repository is the **sole L.ai project**. Previously separate repos
(`Laverna`, `CID`, `CID-Bridge`, `Athena-`) are now archived and merged here.

## Persona

The work has a throughline you can read in a chart, cast for **14 April 1994,
20:09 CDT, Saint Croix Falls, WI** (Lahiri sidereal):

- **Libra ascending, Jupiter in the ascendant** — the measured counselor; weigh
  every claim on a balance.
- **Saturn in Aquarius** — the systems architect; deterministic structure over hype.
- **Sun in Aries (Ashwini)** — the first-mover.
- **Moon in Taurus (Rohini)** — plain-spoken, no drama.

So L.ai speaks plainly, builds on structure, and refuses to guess.

## Workspace layout

```
lai/
  proof/      L.ai · Proof  (Rust, crate laverna → unified `lai` binary)
  gate/       L.ai · Gate   (Rust, crate lai-gate + cid-wasm)
  athena/     L.ai · Athena (Rust, crate athena — lib only, merged into `lai`)
  bridge/     L.ai · Bridge (Node/TypeScript)
  bin/        vendored llama.cpp engine + model drop-in
```

## Build

```bash
# Whole workspace
cargo build --workspace

# Unified binary (Proof + Gate + Athena)
cargo build -p laverna                      # default features
cargo build --release -p laverna            # release

# Individual crates (library only)
cargo build -p lai-gate                     # Gate
cargo build -p athena --no-default-features # Athena (lib check only)

# WebAssembly (Proof + Gate)
cargo build --release --target wasm32-unknown-unknown -p laverna-wasm
cargo build --release --target wasm32-unknown-unknown -p lai-gate-wasm

# Bridge (Node)
cd bridge && npm install
```

## Quick start — offline assistant (Proof + local LLM)

```bash
cargo build --release --features "mcp websearch budget llm milp graph"
./scripts/get-model.sh          # fetch a small model (one command)
./target/release/lai mcp        # talk to it over MCP
```

## Athena — relational reasoning (30 subcommands)

```bash
cargo build --release -p laverna
./target/release/lai athena info                    # system info
./target/release/lai athena wheel --domain aries    # zodiac wheel
./target/release/lai athena classify mercury        # token classification
./target/release/lai athena descent "add two nums"  # 7-layer descent
./target/release/lai athena solve "add 2 3"         # full Bankai pipeline
```

No model? Proof still answers from the verified engine and tells you it did —
it never fabricates.

## Tests & CI

```bash
cargo fmt -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI builds the workspace, both WASM crates, and the Bridge, and attaches static
release binaries to tagged releases.

## License

Apache-2.0 for the whole umbrella. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
Copyright 2026 nutypebuddha.
