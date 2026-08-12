# Changelog

All notable changes to L.ai are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- CI `assistant-termux` job so the `termux`-gated assistant configuration stays
  compile-clean alongside the default build (resolves the untrue T56 ledger claim).
- `CONTRIBUTING.md`, `SECURITY.md`, and `CHANGELOG.md` (referenced by
  `docs/BRAND.md`).

### Changed
- Release CI now builds with the `assistant-web` feature, enabling `ureq/rustls`
  so release artifacts can reach HTTPS model endpoints (previously TLS was absent
  from the release feature set).
- License consolidated to **Apache-2.0** across the umbrella: `gate/LICENSE` and
  `bridge/LICENSE` were Unlicense and are now Apache-2.0; `NOTICE` provenance
  notes updated.

### Fixed
- Removed `ChartSnapshot::default()`, which called `SystemTime::now()` in an
  output-affecting path (a determinism breach of the AGENTS.md rule). Callers must
  now pass a Julian Date explicitly via `ChartSnapshot::new(jd)`.
