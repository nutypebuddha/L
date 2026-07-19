<div align="center">

# L.ai

**Offline-first verification for AI. Verify, don't trust.**

[![CI](https://github.com/nutypebuddha/L/actions/workflows/ci.yml/badge.svg)](https://github.com/nutypebuddha/L/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange)](https://rust-lang.org)
[![WASM](https://img.shields.io/badge/WASM-supported-purple)](#webassembly)
[![Offline](https://img.shields.io/badge/Offline-First-brightgreen)](#offline-first)

</div>

---

**L.ai** is a single binary that does four things — all deterministic, all fail-loud, no network required at runtime:

| | Component | What it does |
|---|-----------|--------------|
| 🔍 | **Proof** | Deterministic reasoning engine: NAND-to-verify cascade, embedded corpus, machine-checkable proof objects, local LLM assistant (MCP) |
| 🛡️ | **Gate** | Per-token validation for LLM output — math, logic, fact, fallacy, bias. ~630KB pure Rust + WASM |
| 🔗 | **Bridge** | Universal MCP bridge — any chatbot (Claude, GPT, Grok, Mistral) hooks into Gate validation through one endpoint |
| 🌐 | **Athena** | Relational reasoning engine — cross-domain formula graph, 30+ subcommands |

## Quick start

```bash
# Build the unified binary
cargo build --release -p laverna

# Validate an expression
./target/release/lai validate "9.11 < 9.9"

# Run the deterministic engine
./target/release/lai tanto eval "2 + 3 * 4"

# Start the MCP assistant (stdio transport)
./target/release/lai assistant --mcp

# Gate: validate LLM output tokens
./target/release/lai gate validate "The earth is flat"
```

No model? The engine still answers from the verified corpus and tells you it did — **it never fabricates.**

## Features

```toml
[dependencies]
laverna = { version = "0.4", features = [
    "assistant",    # Local LLM assistant (ollama/OpenAI-compatible)
    "mcp",          # MCP stdio transport
    "websearch",    # Deterministic web search
    "budget",       # Budget-constrained strategizer
    "llm",          # LLM integration
    "milp",         # Mixed-integer linear programming
    "graph",        # Graph-based reasoning
]}
```

## Architecture

```
lai/
├── proof/      L.ai · Proof  → unified `lai` binary
├── gate/       L.ai · Gate   → per-token validation (Rust + WASM)
├── athena/     L.ai · Athena → relational reasoning (30 subcommands)
├── bridge/     L.ai · Bridge → MCP bridge (Node.js/TypeScript)
└── android-app/              → Android APK (stdio MCP daemon)
```

### NAND-to-verify cascade

Every answer goes through the same path:

```
Input → Tokenize → Gate (per-token) → Proof (NAND core) → Corpus (1,606 facts) → Output
                                         ↓
                                    Refuse if unverifiable
```

No network. No stochastic generation. No confidence scores — just pass/fail with diagnostics.

## WebAssembly

```bash
# Proof + Gate as WASM (~630KB)
cargo build --release --target wasm32-unknown-unknown -p laverna-wasm
cargo build --release --target wasm32-unknown-unknown -p lai-gate-wasm
```

## Android

The Android APK runs L.ai as an MCP daemon over stdio — zero network, zero localhost sockets:

```bash
# Build for Android (requires NDK)
cd android-app && ./gradlew assembleDebug
# Output: android-app/app/build/outputs/apk/debug/app-debug.apk
```

## Testing

```bash
cargo test --workspace           # 625+ tests
cargo clippy --all-targets       # Lint
cargo fmt -- --check             # Format check
```

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

<div align="center">

*Verify, don't trust.* — [L.ai](https://github.com/nutypebuddha/L)

</div>
