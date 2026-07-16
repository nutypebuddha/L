# L.ai — Agent Instructions

Offline, deterministic, fail-loud verification umbrella for AI. One repo, four
functions. Public mark: **L.ai** (tagline: *Verify, don't trust*).

## Workspace

| Function | Path | Crate name | Notes |
|----------|------|------------|-------|
| L.ai · Proof | `proof/` | `laverna` | Main product. Vedic reasoning + proof cascade. |
| L.ai · Gate | `gate/` | `lai-gate` (lib: `cid`) | Per-token validation. WASM target in `gate/cid-wasm/`. |
| L.ai · Athena | `athena/` | `athena` | Relational intelligence engine. Own `AGENTS.md`. |
| L.ai · Bridge | `bridge/` | — | Node/TypeScript. Not a Cargo member. |

Root `Cargo.toml` is a virtual workspace: `[proof, gate, athena, proof/laverna-wasm, gate/cid-wasm]`.

## Dev cycle — run in this order

```bash
cargo fmt -- --check                                          # 1. formatting
cargo clippy --workspace --all-targets -- -D warnings         # 2. lints (must pass)
cargo test --workspace                                        # 3. tests
```

**Proof-specific CI gate** (feature-gated, stricter):
```bash
cargo clippy -p laverna --features "graph,milp,llm" -- -D warnings
cargo test -p laverna --lib --features "graph,milp"
cargo test -p laverna --lib --features llm
```

Single test: `cargo test -p laverna --lib <module>::<test_name>`

## Feature flags (Proof — `proof/Cargo.toml`)

| Flag | Enables | Default |
|------|---------|---------|
| `mcp` | MCP server + websearch | no |
| `websearch` | World Bank stats (ureq) | via `mcp` |
| `budget` | Token budget tracking | no |
| `llm` | Stub inference marker (no heavy deps) | no |
| `milp` | MILP solver (good_lp/microlp) | no |
| `graph` | Graph algorithms (petgraph) | no |
| `bench` | Criterion harness | no |

**Gate** has one optional feature: `proxy` (ureq).

## Critical: Determinism rule

**Every** output path touching a `HashMap` (or any unordered collection) must
sort by a stable key before printing/aggregating. This is a **correctness bug**,
not a style issue. Applies to: `--explain` trace, scoring aggregation, petgraph
results, `domain_graph` neighbors, any future code path.

## Environment

- aarch64 Linux, proot-distro Debian on Android/Termux
- Workspace root: `/root/Laverna/`
- Toolchain: stable, targets include `aarch64-unknown-linux-{gnu,musl}` and `x86_64-unknown-linux-{gnu,musl}`
- `CARGO_BUILD_JOBS` is NOT hardcoded — set per-invocation
- Check disk before building: `df -h / | tail -1` (abort if < 2 GB)

## Export (static builds)

```bash
proof/scripts/export.sh           # builds x86_64-musl static, copies to /sdcard/Download/Laverna/bin/
```

Or manually:
```bash
cargo build --release --target x86_64-unknown-linux-musl -p laverna --features "mcp websearch budget llm milp graph"
cp proof/target/x86_64-unknown-linux-musl/release/laverna /sdcard/Download/Laverna/bin/laverna-x86_64
```

`/sdcard` is vfat FUSE — no symlinks, no exec bits. Use `cp`, never `cp -a`.

## Ticket intake

Ticket files live in `~/downloads` (`/data/data/com.termux/files/home/downloads/`).
```bash
proof/scripts/tickets.sh                       # list ticket files
TICKETS_DIR=/some/dir proof/scripts/tickets.sh # override source
```
"Scan tickets" = list `~/downloads/*.md` and read newest/relevant.

## License

Apache-2.0, sole author `nutypebuddha`. New source files:
```rust
// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0
```
`Cargo.toml` license fields, `LICENSE`, and `NOTICE` must stay in agreement.

## Conventions

- Pure functions only: no global state, deterministic, all inputs as params.
- Commits: Conventional Commits (`feat(gate):`, `fix(proof):`).
- Errors: `anyhow` at call sites, `thiserror` for library types.
- Known-issues tracked in `KNOWN_ISSUES.md` (repo root).

## Gotchas

- **Gate lib name is `cid`** (not `lai-gate`) — `use cid::...` in Rust code.
- **Proof `build.rs`** embeds corpus at compile time — formula TOML edits require rebuild.
- **petgraph HashMap ordering** — always sort results before output.
- **No `.github/workflows/` yet** — CI commands in this file are what to run locally.
