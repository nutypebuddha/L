# Contributing to L.ai

Thanks for your interest in L.ai. This project is **offline-first, deterministic
verification for AI** — *verify, don't trust*. Contributions that strengthen that
contract are welcome.

## Ground rules

- **Pure functions only.** No global mutable state. Every output path is a
  deterministic function of its inputs. If a value comes from a `HashMap` (or any
  unordered collection), sort by a stable key *before* printing or aggregating —
  this is a correctness rule, not a style preference.
- **Fail loud, never fabricate.** A path that cannot verify must return a typed
  error or a refusal, never a guessed answer. See `AGENTS.md`.
- **Determinism.** The flagship claims (chart reproducibility to 0.01°, NAND
  proof cascade) are tested against independent oracles. Do not introduce
  wall-clock, `rand`, or other nondeterminism into an output-affecting path.

## Workflow

1. Fork and branch from `main`.
2. Make your change. Match the existing style; run `cargo fmt` before committing.
3. Commit with [Conventional Commits](https://www.conventionalcommits.org/):
   `feat(gate):`, `fix(proof):`, `docs:`, etc.
4. Open a PR. CI runs the full gate (below); a maintainer reviews.

## Local dev gate (run in this order)

```bash
cargo fmt -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The `assistant` crate has a `termux` feature (Termux system actions) and a `web`
feature (outbound web tools). Both configurations must stay compile-clean; CI
enforces this:

```bash
cargo clippy -p assistant --features termux --all-targets -- -D warnings
cargo test  -p assistant --features termux
cargo fmt   -p assistant -- --check
```

## License

By contributing, you agree your contributions are licensed under **Apache-2.0**
(see `LICENSE`, `NOTICE`). New source files should carry:

```rust
// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0
```

See `AGENTS.md` for the full architecture, feature flags, and conventions.
