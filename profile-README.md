<!--
  Publish this file as the README of the `nutypebuddha/nutypebuddha` repo
  (your GitHub profile). Kept here in the L.ai repo so the brand stays in sync.

  GitHub UI settings (not file-based) to complete the look:
  - Profile picture: the witch-hat Ł glyph (docs/brand-LAI.svg).
  - Banner: a wide crop of docs/brand-LAI.svg or docs/ecosystem.svg.
  - Pinned repos (Settings → Profile → Pinned): 1) L  2) Athena (archived)
    3) best Rust lib  4) a CLI/tool  5) experimental AI  6) a demo/showcase.
  - First thing visitors see is this README's header block below.
-->

<div align="center">

<img src="https://raw.githubusercontent.com/nutypebuddha/L/main/docs/brand-LAI.svg" width="110" height="146" alt="L — Old English L with witch hat"/>

# Ł  L

**Building L — a Rust-native AI ecosystem focused on autonomous tooling, developer workflows, and local-first intelligence.**

[![L.ai](https://img.shields.io/badge/Flagship-L.ai-7df9ff)](https://github.com/nutypebuddha/L)
[![Rust](https://img.shields.io/badge/Rust-2021-c2410c)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue)](https://github.com/nutypebuddha/L/blob/main/LICENSE)

**Verify, don't trust.** — offline-first, deterministic, fail-loud.

</div>

---

## The platform

L is not a single repo — it's a growing platform. One mark, one contract:
**deterministic verification instead of probabilistic trust.**

```
L
├── Core        L.ai · Proof / Gate / Bridge / Compute  (the verification substrate)
├── Athena      relational reasoning engine  [archived reference]
├── CLI         lai — one binary, four functions
├── SDK         lai-core — shared domain types + error hierarchy
├── Plugins     MCP tools, LLM adapters, validators
└── Examples    WASM playground, Android daemon, demos
```

### How it connects

```
            Ł  L  (identity / umbrella)
                 │
          ┌──────┴──────┐
          │             │
    Athena [archived]   Mana Core [archived]
          │             │
          ├─────────────┤
        L.ai Core  (offline verification substrate)
          │
   ┌──────┼──────────────┐
 CLI      SDK            Plugins
(lai)   (lai-core)     (MCP / adapters / validators)
```

## Pinned repositories

| # | Repository | Why it's pinned |
|---|------------|-----------------|
| 1 | **[L](https://github.com/nutypebuddha/L)** | Flagship — the whole ecosystem in one repo. |
| 2 | **Athena** | The relational reasoning engine that started it. *(archived)* |
| 3 | **best Rust library** | Demonstrates engineering quality (e.g. `lai-core`). |
| 4 | **CLI / tooling** | `lai` — one binary, four functions. |
| 5 | **Experimental AI** | Cutting-edge local-first intelligence work. |
| 6 | **Showcase / demo** | A runnable demo or WASM playground. |

## Mission

> Building L — a Rust-native AI ecosystem focused on autonomous tooling,
> developer workflows, and local-first intelligence.

## Latest

- 🚀 **L.ai v0.4.1** — `lai-core` consolidation, repo hygiene, Athena eval memoization. [Release](https://github.com/nutypebuddha/L/releases/tag/v0.4.1)
- 🧪 Every crate ships `cargo test` + clippy under `-D warnings`.

## Contact

- GitHub: [@nutypebuddha](https://github.com/nutypebuddha)
- Issues & discussions: open one on any ecosystem repo.

---

<div align="center">

*Verify, don't trust.* — the L ecosystem

</div>
