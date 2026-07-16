# Changelog

All notable changes to L.ai · Proof (Laverna) will be documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Fixed
- `strategize` exponential blowup on `--budget >20` — added LP-relaxation upper bound for branch-and-bound pruning (T52). Budget=30 now completes in ~0.6s (was: 30+ min / NODE_CAP error).
- `schema domain` template missing `[scoring.score_cool]` table, `cool` item, and `score_cool` in `objective.maximize` — template was broken as documented (T53). Also fixed stale flags in header comment (`--lat`/`--lon` → `--latitude`/`--longitude`).
- Corpus entity `muladhara` description contained Chinese text `安全感` — replaced with "security".
- Clippy: too-many-arguments in `optimize` module — refactored `enumerate_attributes`/`enumerate_perks` to use `SolverState` struct.
- Clippy: 19 additional lints across `gate` crate — clamp, sort-by-key, approx-const, loop-index, identical-blocks, vec-init-then-push fixes.

### Changed
- Monorepo restructure: proof, gate, athena merged into single `nutypebuddha/lai` workspace (commit `d9cf82d`).
- All 7 predecessor repos archived + redirected to `nutypebuddha/lai`.
- Profile bio updated: "Building L.ai — deterministic proof, verification, reasoning. Verify, don't trust."

## [0.3.0] - 2026-07-15

### Added
- `corpus` subcommand: `export`, `validate`, `diff`, `graph`, `lint`.
- Overlay loader (`~/.laverna/corpus/`, `./corpus`) for user TOML files.
- Versioned corpus: `CORPUS_VERSION` + `CORPUS_CONTENT_HASH` via build.rs.
- `Profile` trait + stubs `WuXingProfile`, `TemperamentProfile`.
- `--format json` for `info`, `entity-get`, `formulas`, `entities`.

### Fixed
- `chart` lagna duplicated name → prints symbol ` Scorpio`.
- `route` silent ignore when both `--query` and `--repos` given → now warns.
- `websearch` compound-query mis-segmentation, multi-year hang → ISO-3166 gazetteer + 10s timeout.

## [0.2.0] - 2026-07-14

### Added
- MCP protocol bumped to `2025-11-25`; `route` + `build` tools (9 total).
- `solve --proof-out <path>` + `laverna verify <path>` for proof objects.
- Vendored llama.cpp engine with auto-detect + fallbacks.

### Fixed
- `chart` lagna prints symbol.
- `route` warns on ambiguous flags.
