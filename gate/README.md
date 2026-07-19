<div align="center">

# Ł L.ai · *Gate*

**Per-token validation for LLM output — verify, don't trust.**

> Part of the **[L.ai](https://github.com/nutypebuddha/lai)** ecosystem.

<img src="../docs/brand-LAI.svg" width="72" height="96" alt="L.ai witch-hat Ł glyph"/>

</div>

---

# CID — Calibrated Inference Device

> **L.ai · Gate** — part of the [L.ai](https://github.com/nutypebuddha/lai) ecosystem. *Verify, don't trust.*

**A per-token validation layer for LLMs using pachinko mechanics to enforce mathematical, logical, and factual constraints at inference time.**

> In the unified `lai` binary, Gate is invoked as `lai gate <subcommand>`. The standalone `cid` crate is the same engine, lib name `cid`.

[Features](#features) • [Installation](#installation) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing)

---

## Overview

CID acts as a validation gate between LLM output and the real world. Think of LLMs as a river and CID as the dams and structure to maximize them.

```
┌─────────────────────────────────────────────────────────────┐
│                    CID Validation Pipeline                    │
├─────────────────────────────────────────────────────────────┤
│  LLM Output → [Math Gate] → [Logic Gate] → [Fact Gate]     │
│                    ↓              ↓              ↓           │
│              [Confidence] → [Fallacy] → [Bias Detection]   │
│                              ↓                              │
│                    Validated Output                          │
└─────────────────────────────────────────────────────────────┘
```

## Features

### Core Validation Gates

| Gate | Description | Patterns |
|------|-------------|----------|
| **Math** | Equation validation & auto-fix | Arithmetic, algebra, units |
| **Logic** | Premise/conclusion checking | Deductive, inductive reasoning |
| **Fact** | Knowledge base lookup | 776+ facts across 12 domains |
| **Confidence** | Platt scaling calibration | Domain-specific thresholds |
| **Fallacy** | Logical fallacy detection | 69 patterns, 14 types |
| **Bias** | Cognitive bias detection | 43 patterns, 12 types |
| **Formal** | Formal verification | Symbolic logic |

### Advanced Features

- **Auto-Fix**: Corrects math errors, typos (200+), unit conversions, code consistency
- **Streaming Validation**: Real-time SSE events for token-by-token validation
- **Multi-Provider Proxy**: OpenAI, Anthropic, Gemini, and generic endpoints
- **Semantic Cache**: Avoid redundant LLM calls with meaning-based caching
- **Prompt Compression**: 30-50% token reduction before LLM calls
- **Response Scoring**: Quality evaluation without re-querying
- **MCP Server**: 22 tools for AI agent integration

### Performance

- **Binary Size**: ~615KB (WASM) / ~872KB (native)
- **Validation Overhead**: <0.2% vs LLM inference cost
- **Zero Dependencies**: Pure Rust with `std` only (optional `ureq` for HTTP)

## Installation

### From Source

```bash
# Clone the repository
git clone https://codeberg.org/NutypeBuddha/cid.git
cd cid

# Build with cargo
cargo build --release

# Binary is at target/release/cid
./target/release/cid --help
```

### With WASM Support

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Build WASM binary
cd cid-wasm
./build.sh

# Output: cid-wasm/target/wasm32-unknown-unknown/release/cid_wasm.wasm (615KB)
```

### On Termux (Android)

```bash
# Install Rust
pkg install rust

# Clone and build
git clone https://codeberg.org/NutypeBuddha/cid.git
cd cid
cargo build --release
```

## Quick Start

### CLI Usage

```bash
# Validate math expression
echo "2 + 3 = 5" | cid validate --- math

# Fix math errors
echo "2 + 3 = 6" | cid fix --- math

# Fix typos
echo "hte cat sat on teh mat" | cid fix --- 

# Validate with all gates
echo "The Earth is flat" | cid validate --- fact

# Score response quality
cid score "Everyone knows this is true"
```

### MCP Server

```bash
# Start MCP server (stdio)
cid mcp

# Start HTTP MCP server (for Claude web)
cid mcp-http 127.0.0.1:8080
```

### HTTP Proxy

```bash
# Start proxy with OpenAI
cid proxy --port 8080 --llm https://api.openai.com/v1/chat/completions --key sk-...

# Validate via API
curl -X POST http://localhost:8080/v1/validate \
  -H "Content-Type: application/json" \
  -d '{"text": "2 + 3 = 5", "context": "math"}'
```

## Documentation

### Architecture

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Public API
├── core/
│   ├── pin.rs           # Validation gates
│   ├── ball.rs          # Token candidates
│   └── pocket.rs        # Selected token
├── gates/
│   ├── mod.rs           # GateValidator trait
│   ├── math.rs          # Math validation
│   ├── logic.rs         # Logic validation
│   ├── fact.rs          # Fact validation
│   ├── confidence.rs    # Confidence scoring
│   ├── fallacy.rs       # Fallacy detection
│   ├── bias.rs          # Bias detection
│   └── formal.rs        # Formal verification
├── inference/
│   ├── pipeline.rs      # Validation pipeline
│   ├── proxy.rs         # HTTP proxy
│   ├── stream.rs        # Streaming validation
│   └── compressor.rs    # Prompt compression
├── mcp/
│   ├── server.rs        # MCP server
│   └── tools.rs         # 22 MCP tools
├── tanto/               # Tanto compute engine
│   ├── math.rs          # Math operations
│   ├── convert.rs       # Unit conversion
│   ├── formulas.rs      # Physics formulas
│   └── solver.rs        # Problem solvers
└── kb/
    └── facts.rs         # Knowledge base (776+ facts)
```

### MCP Tools

| Tool | Description |
|------|-------------|
| `cid_validate` | Validate text through all gates |
| `cid_fix` | Auto-fix math, typos, consistency |
| `cid_lookup` | Look up a fact by name |
| `cid_search` | Search KB by keyword |
| `cid_detect_fallacies` | Detect 14 types of logical fallacies |
| `cid_detect_biases` | Detect 12 types of cognitive biases |
| `cid_sanity_check` | Check numeric values against physical ranges |
| `cid_score` | Score response quality |
| `cid_compress` | Reduce prompt token count |
| `cid_sample` | Request LLM completion |
| `cid_tanto_eval` | Evaluate math expressions |
| `cid_tanto_convert` | Convert between units |
| `cid_tanto_solve` | Solve physics problems |

### Knowledge Base Domains

| Domain | Symbol | Facts |
|--------|--------|-------|
| Math & Logic | α | Constants, formulas, theorems |
| Physics & Chemistry | β | Physical constants, elements |
| Astronomy | γ | Celestial bodies, distances |
| Earth & Environment | δ | Geographic, climate data |
| Biology & Medicine | ε | Biological facts, medical data |
| CS & AI | ζ | Algorithms, ML concepts |
| Engineering | η | Technical specifications |
| Economics & Finance | θ | Market data, financial ratios |
| History | ι | Historical events, dates |
| Language | κ | Linguistic facts |
| Philosophy | λ | Philosophical concepts |
| Psychology | μ | Psychological phenomena |

## Configuration

### Environment Variables

```bash
# Model routing
export CID_DEFAULT_MODEL="anthropic/claude-sonnet-4-6"
export CID_SMALL_MODEL="openai/gpt-4o-mini"
export CID_REASONING_MODEL="anthropic/claude-sonnet-4-6"

# Provider priority
export CID_PROVIDER_PRIORITY="anthropic,openai,google,ollama"

# Timeouts
export CID_MODEL_TIMEOUT="60000"
```

### Feature Flags

```toml
[dependencies]
cid = { version = "0.8.0", features = ["proxy"] }

# Available features:
# - proxy: HTTP client for LLM proxying
```

## Performance Benchmarks

| Operation | Time | Cost |
|-----------|------|------|
| Math gate | ~0.001ms | ~$0.000001 |
| Logic gate | ~0.002ms | ~$0.000002 |
| Fact gate | ~0.001ms | ~$0.000001 |
| Confidence | ~0.0005ms | ~$0.0000005 |
| **Total** | **~0.0045ms** | **~$0.0000045** |

**Overhead vs GPT-4o**: 0.18% ($0.0045/Mtok vs $2.50/Mtok)

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_math_validation

# Run benchmarks
cargo bench
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone
git clone https://codeberg.org/NutypeBuddha/cid.git
cd cid

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## License

This project is in the public domain. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by pachinko mechanics for validation routing
- Built with pure Rust for maximum portability
- Tanto compute engine for math operations

---

**Made with ❤️ by NutypeBuddha**
