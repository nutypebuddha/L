# Laverna — Agent Instructions

Vedic reasoning engine reboot. 4-layer architecture:
**Asauchi** → **Zanpakuto** → **Shikai** → **Bankai**.
NAND gate primitives at the bottom. Determinism-first.

## Environment
- aarch64 Linux; check disk before building: `df -h / | tail -1`
- `CARGO_BUILD_JOBS` is NOT hardcoded — set per-invocation
- `/sdcard` is vfat FUSE: no symlinks, no exec bits, use `cp`

## Dev cycle
```bash
cargo clippy -- -D warnings && cargo test --lib && cargo fmt -- --check
```

## CI gate order
`fmt --check` → `clippy -D warnings` (default + `--features llm`) →
`cargo deny check` → `cargo test` → `cargo test --features llm --lib` → `cargo audit`

## Build & features
```bash
cargo build --release                                     # native
cargo build --release --target x86_64-unknown-linux-musl --no-default-features
```
| feature | enables | default |
|---------|---------|---------|
| `mcp` | rmcp + tokio JSON-RPC server | no |
| `websearch` | ureq (World Bank stats) | via `mcp` |
| `budget` | token budget tracking | no |
| `bench` | criterion harness | no |
| `llm` | llama-gguf local LLM backend | no |
| `portable` | embed corpus in binary | no |

## Architecture
- **Layer 0 — Primitive**: `src/primitive/`, `src/descent/`, `src/gyro/`
- **Layer 1 — Asauchi**: `src/asauchi/`, `src/formula/`, `src/entity/`, `src/ephemeris/`, `src/chart/`
- **Layer 2 — Zanpakuto**: `src/zanpakuto/`, `src/shikai/`
- **Layer 3 — Bankai**: `src/bankai/`, `src/mcp/`

Pipeline: query → Zanpakuto::NLP → DescentEngine → Shikai::process → Bankai::solve

## Conventions
- Formulas, not facts: encode relationships, not static lookups
- Cross-domain by default: new formulas reference ≥2 grahas
- Commits: Conventional Commits (`feat(wheel):`, `fix(bankai):`)
- Errors: `anyhow` at call sites, `thiserror` for library types
