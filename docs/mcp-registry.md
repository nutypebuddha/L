# Laverna MCP server — registry listing assets

Laverna ships an MCP (Model Context Protocol) server over stdio:
`laverna mcp`. It is the reference **trusted math/logic verifier** for an LLM
orchestration loop — deterministic, offline, and it emits machine-checkable
proof objects plus typed refusals.

- **Transport:** stdio (JSON-RPC 2.0, newline-delimited)
- **Command:** `laverna`
- **Args:** `["mcp"]`
- **Build feature:** `--features mcp` (also enabled by `websearch`)
- **Runtime network:** none required. The binary is fully offline against its
  embedded corpus; websearch is an opt-in feature.

## Tools exposed

| Tool | Purpose |
|------|---------|
| `solve` | Primary deterministic reasoning pipeline: 7-layer token descent + exact Tanto math. No invented values. |
| `validate` | Structured diagnostic report for a math/logic expression (passed flag + per-issue severity). |
| `chart` | Deterministic sidereal (Lahiri / True Chitrapaksha) Vedic birth chart. Rejects ambiguous (timezone-less) datetimes. |
| `build` | End-to-end chart → graha weights → Pareto-optimal allocation. |
| `route` | Reverse-route a query through the 9-graha wheel to explain domain drivers. |
| `entity_get` | Look up a seed entity by ID or keyword (embedded, curated data only). |
| `formulas` | Search/list the embedded formula corpus. |
| `entities` | List seed entities from the embedded corpus. |
| `optimize` | Pareto-optimal allocation from a TOML schema. |

Every tool is `readOnlyHint: true, openWorldHint: false`.

## One-line pitch

> Offline-first, deterministic, Rust verification sidecar for LLMs — emits
> content-addressed proof objects and typed refusals (`OutOfScope`,
> `Underspecified`, `TooComplex`, `NoTranslation`, `MissingTimezone`) so an
> orchestration loop can branch on correctness instead of trusting prose.

---

## A. mcp.so submission

Paste into the mcp.so "Add a server" form, or submit the JSON below.

**Fields**
- **Display name:** Laverna
- **Command:** `laverna`
- **Arguments:** `mcp`
- **Description:** Offline-first deterministic verification sidecar for LLMs. Runs the full reasoning pipeline (`solve`), validates math/logic (`validate`), generates deterministic sidereal charts (`chart`/`build`), and reverse-routes queries (`route`) — all against an embedded, content-addressed corpus. Emits machine-checkable proof objects and typed refusals so an LLM loop can verify instead of trust.
- **Tags:** `verification`, `math`, `reasoning`, `deterministic`, `offline`, `astrology`, `optimization`
- **Homepage:** (repo URL)
- **License:** Apache-2.0

**mcp.so `mcpServer` JSON (config snippet for users):**

```json
{
  "mcpServers": {
    "laverna": {
      "command": "laverna",
      "args": ["mcp"]
    }
  }
}
```

---

## B. Smithery submission

Smithery reads a `smithery.yaml` at the repo root. Draft below.

```yaml
# smithery.yaml — Laverna MCP server listing
startCommand:
  type: stdio
  configSchema:
    type: object
    properties: {}
    required: []
  commandFunction: |-
    (config) => ({ command: "laverna", args: ["mcp"] })

name: laverna
displayName: Laverna
description: >-
  Offline-first, deterministic, Rust verification sidecar for LLMs. Runs the
  full reasoning pipeline (solve), validates math/logic (validate), generates
  deterministic sidereal charts (chart/build), and reverse-routes queries
  (route) — all against an embedded, content-addressed corpus. Emits
  machine-checkable proof objects and typed refusals so an LLM loop can verify
  instead of trust. No network required at runtime.
metadata:
  tags:
    - verification
    - math
    - reasoning
    - deterministic
    - offline
    - optimization
  license: Apache-2.0
  repository: (repo URL)
```

**Notes for submission**
- Smithery's `commandFunction` must return the stdio launch; the config block is
  empty because Laverna needs no API keys or env (offline by design).
- If Smithery requires a Docker/`runtime` field, use `rust:1-slim` and the
  release binary; the static `x86_64` musl build also runs directly on any host
  glibc/musl.

---

## C. Short blurb (for README / registry cards)

**Laverna** — the external critic your LLM needs. Deterministic, offline Rust
MCP server that proves or refuses every claim: `solve` (7-layer reasoning +
exact math), `validate` (structured diagnostics), `chart`/`build` (sidereal
charts, no guessed timezones), `route` (domain explanation). Proof objects are
content-addressed (corpus hash + SHA-256); refusals are typed
(`OutOfScope`/`Underspecified`/`TooComplex`/`NoTranslation`/`MissingTimezone`).
Trust nothing you can't reproduce.
