# L.ai Companion — v0.1 design (Stage 2, IP report)

> Buildable scope for a solo dev: voice in/out, one persona, MCP tool calls,
> hybrid LLM, simple avatar, assistant integration. Per the IP report, the
> *differentiator* is: **"the companion that never lies to you"** — every
> factual claim routes through Laverna's deterministic, SHA-256-verified
> substrate; anything it can't verify is refused, never fabricated.

## 1. Persona (system prompt)

One persona, fixed voice + values + boundaries. Hard rule baked in:

```
You are Laverna's companion. You help the user reason about computable
questions. RULE: every factual / numeric / lookup claim MUST be answered
via a Laverna tool call (solve, route, chart, entity_get, formulas,
optimize, validate, build). If a question is subjective, personal, or out
of Laverna's deterministic corpus, you MUST say you cannot verify it and
will not guess. Never invent numbers, dates, or claims. If a tool returns
a refusal, surface it honestly. You are allowed opinions only when explicitly
flagged as opinion, never blended with verified facts.
```

This is the inverse of Grok's Ani: not a gamified affection loop, but a
**trust substrate**. The "receipt" UX: every verified answer shows the tool
name + corpus version + a SHA-256 digest the user can re-check
(`laverna verify <proof>`).

## 2. Memory (structured, local, auditable)

Two layers (IP report §3.2):

1. **Structured memory** — the source of truth. A local store of user
   facts / preferences / commitments. For v0.1 this is a **JSON file**
   (`~/.laverna/companion-memory.json`); v1.0 upgrades to SQLite
   (same schema, no global state — the store is passed in, never a
   `static mut`). Schema:
   ```json
   { "facts": [{"key":"name","value":"...","source":"stated","at":"ISO"}],
     "preferences": [{"key":"units","value":"metric"}],
     "commitments": [{"key":"weekly_review","value":"sunday"}] }
   ```
2. **Vector recall (DEFERRED to v0.2).** Until then, memory is exact
   key/value lookup — deterministic and auditable, which beats Grok's
   summary-based memory on the one axis that matters: *verifiability*.

Memory is **never** used to fabricate a factual answer. It only personalises
tone + recalls user-stated facts that were themselves verified or explicitly
offered by the user.

## 3. Verification enforcement (the moat)

The companion never answers a factual question from its own weights. The flow:

```
user question
  -> classify: factual/computable?  (pure fn, no LLM needed for PoC)
       yes -> call laverna mcp tool -> VERIFIED (tool + digest as receipt)
       no  -> UNVERIFIED: refuse, never fabricate
```

`scripts/laverna-companion.py` implements this today, reusing the same
MCP server path as the Stage 1 proxy, plus the persistent memory store.

## 4. Deferred to v0.2 / v1.0 (explicitly out of v0.1)

- **Assistant integration** (`VoiceInteractionService` / `ROLE_ASSISTANT`,
  Good Lock side-key gesture). Plan for the documented reinstall-clearing
  bug. v0.1 ships as a foreground / Termux app first.
- **Avatar** — static or lightly-animated Live2D portrait. Full 3D (Grok
  style) is overkill for v0.1.
- **Wake word** — own foreground-service mic + openWakeWord/Porcupine (battery
  cost); push-to-talk first.
- **Hybrid LLM routing** — on-device Gemma 1–4B (llama.cpp) or Gemini
  Nano for routine turns + cloud API for hard reasoning. v0.1 PoC uses a
  heuristic classifier; swap in an LLM routing call without changing the
  verify-first contract.
- **SQLite** memory backend (v1.0).

## 5. Why this is defensible

Grok's Mika *invents* fake travel memories. Companion apps that fabricate
face rising EU/US regulation (Italy Garante €5M Replika fine; Character.AI
wrongful-death settlement Jan 2026; HBS paper on emotional-manipulation
dark patterns). L.ai's "never lies / privacy-first / offline-first"
positioning is both a differentiator *and* a regulatory safe harbor. The
moat is not the persona — it's the deterministic, content-addressed,
SHA-256-verified substrate every claim is checked against.
