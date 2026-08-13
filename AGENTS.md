# L.ai — Agent Instructions

Offline, deterministic, fail-loud verification umbrella for AI. One binary
(`lai`), multiple functions. Tagline: *Verify, don't trust*.

## Workspace (Cargo virtual workspace, root `Cargo.toml`)

| Path | Crate | Binary | Notes |
|------|-------|--------|-------|
| `lai-core/` | `lai-core` | — | Shared domain types + `LaiError` hierarchy. Base dep of everything. |
| `proof/` | `laverna` | `lai` | Main product. Vedic reasoning + NAND-to-verify cascade. |
| `gate/` | `lai-gate` (lib: `cid`) | — | Per-token validation. WASM in `gate/cid-wasm/`. |
| `athena/` | `athena` | (lib) | Relational reasoning (30+ subcommands). Linked into `lai athena`. |
| `proof/laverna-wasm/`, `gate/cid-wasm/` | wasm crates | — | `wasm32-unknown-unknown` targets. |
| `assistant/` | `assistant` | — | Optional dep of `laverna`. Voice-first assistant. |
| `bridge/` | — | — | Node/TypeScript. **Not** a Cargo member; built with `npm`. |

`athena` is a library dependency of `proof` (optional `mcp`/`llm`/`budget` deps
gated by feature). Standalone `athena` binary is `athena/src/main.rs.standalone`
(not built by default; use `lai athena`). Gate's Rust lib is named **`cid`**
(`use cid::...`), not `lai-gate` — `lai-gate` is only the `Cargo.toml` name.

## The `lai` binary (`proof/`)

Top-level subcommands include `validate`, `mcp`, `score`, `verify`, `corpus`,
`gate`, `gaterepl`, `tanto`, `athena`, `companion`, and `assistant` (only when
the `assistant` feature is on). For the current list run `lai --help` /
`lai <sub> --help` — don't trust prose.

## Dev cycle — match CI exactly (`.github/workflows/ci.yml`)

Run in this order. CI fails on any warning (`RUSTFLAGS="-D warnings"`):

```bash
cargo fmt -- --check                                          # 1. formatting
cargo clippy --workspace --all-targets -- -D warnings         # 2. lints
cargo deny check                                              # 3. license/bans/advisory (needs cargo-deny)
cargo test --workspace                                        # 4. tests (default features)
cargo test -p laverna --test swiss_oracle                    # 5. Swiss-ephemeris oracle gate
cargo test -p laverna --features mcp --test mcp_parity        # 6. CLI/MCP parity gate
```

`cargo-deny` is a separate tool (CI installs it via `taiki-e/install-action`);
install it locally or step 3 errors. The two `--test` gates are **separate
jobs in CI**, not part of `cargo test --workspace`.

Assistant is gated separately (Termux-only system actions):

```bash
cargo fmt   -p assistant -- --check
cargo clippy -p assistant --features termux --all-targets -- -D warnings
cargo test  -p assistant --features termux
```

Optional stricter Proof gate (local, not in CI):

```bash
cargo clippy -p laverna --features "graph,milp,llm" -- -D warnings
cargo test -p laverna --lib --features "graph,milp"
cargo test -p laverna --lib --features llm
```

Single test: `cargo test -p laverna --lib <module>::<test_name>`

Athena smoke test (after `cargo build --release -p laverna`):

```bash
./target/release/lai athena info
./target/release/lai athena wheel --domain aries
./target/release/lai athena classify mercury
```

## Feature flags (`proof/Cargo.toml`)

`default = ["assistant", "mcp", "websearch", "budget", "milp", "graph"]` — a
plain `cargo build` already pulls heavy deps; use `--no-default-features` for a
minimal build.

| Flag | Enables | Default |
|------|---------|---------|
| `mcp` | MCP server + websearch + athena mcp | yes |
| `websearch` | World Bank stats (ureq) | via `mcp` |
| `budget` | Token budget tracking + athena budget | yes |
| `llm` | Local LLM inference (llama-gguf + ureq) | no |
| `milp` | MILP solver (good_lp) | yes |
| `graph` | Graph algorithms (petgraph) | yes |
| `bench` | Criterion harness | no |
| `assistant` | Voice-first assistant (STT/TTS/intent/actions) | yes |
| `assistant-web` | Assistant outbound web tools (search/fetch) — needs `assistant` | no |

`gate` has one optional feature: `proxy` (ureq).
**`termux`** is a feature of the *`assistant`* crate (not `proof`): it enables
SMS/calls/camera/battery actions and must only be set when the binary runs under
Termux — advertising those tools elsewhere causes runtime failures.

## Critical: Determinism rule

Every output path touching a `HashMap`/`HashSet` (or any unordered collection)
must sort by a stable key before printing/aggregating. This is a **correctness
bug**, not style. Applies to: `--explain` trace, scoring aggregation, petgraph
results, `domain_graph` neighbors, any new path.

- No `SystemTime`/`Instant` in output-affecting code unless the task is timing.
- No silent fallback to `true`/`passed`/`ok` on unparseable input — validators
  fail loud.
- A fix isn't done until a **rebuild is stable across repeated runs** (no
  variance).

## Corpus is embedded at compile time

`proof/build.rs` embeds formula/entity TOML from `proof/formulas|entities`
**and** `athena/formulas|entities` into the binary (self-contained, CWD
independent). **Any corpus TOML edit requires a rebuild** — `cargo build` won't
re-embed unless the dirs change; `touch` them or `cargo clean -p laverna`.
Overlay dirs `~/.laverna/corpus/` or `./corpus/` merge user TOML over seed.

## Build / export

```bash
cargo build --workspace
cargo build --release -p laverna
cd bridge && npm ci || npm install        # bridge (Node) — separate from Cargo

# WASM crates build with cargo; generating JS bindings needs wasm-bindgen-cli:
cargo build --release --target wasm32-unknown-unknown -p laverna-wasm
cargo build --release --target wasm32-unknown-unknown -p lai-gate-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/laverna_wasm.wasm --out-dir proof/laverna-wasm/www --target web

# Static release (CI does this for tags): musl + assistant-web, verified statically linked
cargo build --release --target x86_64-unknown-linux-musl -p laverna \
  --features "mcp websearch budget llm milp graph assistant-web"
proof/scripts/export.sh                    # equivalent; copies to /sdcard/Download/Laverna/bin/
```

Release profile is aggressive (`opt-level="z"`, `lto="fat"`, `panic="abort"`,
`strip`): build in debug for fast iteration.

`/sdcard` is vfat FUSE — no symlinks, no exec bits. Use `cp`, never `cp -a`.

## Assistant (voice-first)

- `assistant` is on by default. Text mode needs no mic:
  `./target/release/lai assistant --text "set a 5 minute timer"`.
- Speech models (`whisper-tiny.en.bin`, `piper-en_US-*.onnx`) are **not** in the
  repo — fetch with `./scripts/get-models.sh`. Offline LLM model via
  `./scripts/get-model.sh`.
- System actions (SMS/calls/camera/battery/location) shell out to `termux-*`
  commands — only work under Termux with the Termux:API app; build with
  `--features termux` (assistant crate).
- Audio capture/playback may not work under proot without PulseAudio/PipeWire
  forwarding — test on real Termux.

## Environment / toolchain

- Targets: `aarch64-unknown-linux-{gnu,musl}`, `x86_64-unknown-linux-{gnu,musl}`.
  Stable Rust.
- `CARGO_BUILD_JOBS` is not hardcoded — set per-invocation.
- Check disk before building: `df -h / | tail -1` (abort if < 2 GB).
- Development is on Android/Termux (proot); the repo root is the checkout dir.

## License

Apache-2.0, sole author `nutypebuddha`. New source files need:

```rust
// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0
```

`Cargo.toml` license fields, `LICENSE`, `NOTICE` must agree. `cargo deny`
(CI step 3) enforces the allowlist in `deny.toml`.

## Conventions

- Pure functions only: no global state, deterministic, all inputs as params.
- Commits: Conventional Commits (`feat(gate):`, `fix(proof):`).
- Errors: `anyhow` at call sites, `thiserror` for library types.
- Known issues: `KNOWN_ISSUES.md` (repo root).
- Tickets: `proof/scripts/tickets.sh` lists `~/downloads/*.md`; `TICKETS_DIR`
  overrides the source.

## See also

- `proof/AGENTS.md` — Proof internals (layers, `optimize` schema, corpus overlay).
- `docs/` — brand/ecosystem docs.
