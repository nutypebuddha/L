# Security Policy

L.ai is an **offline-first verification layer for AI output**. Its security model
rests on a simple invariant: *the engine never trusts model output — it verifies.*

## Reporting a vulnerability

Please report security issues **privately**, not via public GitHub issues.

- Email: security@laverna (PGP encouraged; key published on request)
- Or open a private security advisory on the repository.

Include: affected component (`proof` / `gate` / `athena` / `bridge` /
`assistant`), reproduction steps, and the expected vs. actual behavior. We aim to
acknowledge within 72 hours and propose a fix window with you.

## Trust boundaries

- **No network at runtime by default.** The core verification engine (Proof,
  Gate) requires zero network. The `assistant/web` feature and `llm` feature are
  opt-in and explicitly widen the capability set; they are off in the default
  build for exactly this reason.
- **LLM output is untrusted input.** Anything from a model (local or hosted) is
  treated as unverified. The agent loop must fall back to the deterministic
  engine when the model is unavailable or yields nothing.
- **Hosted model endpoints.** When configured via `LAI_LLM_BASE_URL`, the engine
  talks OpenAI-compatible `/v1/chat/completions`. Credentials are supplied through
  the `LAI_LLM_API_KEY` environment variable (sent as a `Bearer` header) — never
  hard-coded. Use HTTPS endpoints only; the default build has no TLS, so the
  `assistant-web` feature is required for any `https://` base URL.
- **The MCP transport is stdio, not a socket.** The Android app drives the engine
  over owned process pipes — there is no localhost port and no auth surface to
  attack.

## Known hard constraints

- `panic = "abort"` in the release profile: a panic in the MCP server terminates
  the session rather than returning an error frame. Request handlers must return
  `Result`s and never `unwrap()` on malformed input. (Tracked for the deeper
  unwrap audit.)
- The corpus is embedded at compile time (`build.rs`). Editing formula/entity TOML
  requires a rebuild; the embedded content hash is versioned so drift is detected.
