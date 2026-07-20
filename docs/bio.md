# Biography — drop-in versions

Four sizes. Every number below was verified against the repo at HEAD (July 2026):
1,421 passing tests, 64,600+ lines of Rust excluding Athena, ticket ledger through T58.
Name and pronouns are a find-replace away if a venue calls for different ones.

Preferred name: Ashley (NutypeBuddha). Legal name: Ryan Jason Phernetton. Pronouns: she/her.

---

## One-liner (GitHub bio / social, ~150 chars)

Ashley (NutypeBuddha) is a self-taught Rust developer building L.ai — offline-first,
deterministic AI verification you can re-check — on an Android phone. Verify, don't trust.

---

## Short (~70 words — footers, crate metadata, speaker blurbs)

Ashley (NutypeBuddha, legal name Ryan Jason Phernetton) is a self-taught Rust developer
from northern Wisconsin building **L.ai**: an offline-first verification stack for AI —
deterministic proof engine, per-token output gate, universal MCP bridge, and relational
reasoning — written and shipped entirely from an Android phone. Apache 2.0, CI-gated,
public bug ledger. Motto and method: *verify, don't trust.*

---

## Standard (~320 words — grant applications, About pages)

Ashley, known online as NutypeBuddha (legal name Ryan Jason Phernetton), is a self-taught
Rust developer building **L.ai**, an offline-first verification stack for AI. The premise:
language models guess, and infrastructure shouldn't. L.ai answers only what it can prove —
a deterministic reasoning engine (Proof), a per-token validation layer for LLM output
(Gate), a universal MCP bridge so any chatbot can hook into that validation (Bridge), and
a cross-domain formula graph (Athena) — one binary, no network required at runtime, built
to refuse loudly rather than fabricate.

The distinctive part is the workshop. The entire stack — more than sixty thousand lines of
Rust, over 1,400 passing tests — is developed on an Android phone under Termux and a proot
Ubuntu userland, and shipped through a CI gate of rustfmt, clippy, cargo-deny, and the full
test suite. Development runs on adversarial verification: builds are identified by hash
before they're trusted, bugs are filed as numbered tickets in a public KNOWN_ISSUES ledger
(T1 through T58 and counting), and determinism is a hard requirement — identical inputs
produce byte-identical outputs, audited by the binary itself. Everything ships Apache 2.0.

She describes the approach as "NAND-to-Tetris for reasoning": start from primitives you can
prove, build upward, and never invent a scalar you can't derive. The lineage runs through
gate-forge (a logic-gate substrate crate), CID (a calibrated inference device), and Athena
into the unified L.ai. The working ethic comes from John Nash, Louis Armstrong, and Yoh
Asakura — self-taught mastery, done for its own sake.

Off the main line she designs games and LARP systems — two Godot Android games shipped from
the same phone — and is developing Manitoupunk, an aesthetic framework rooted in Anishinaabe
cosmology. She is based in Wisconsin's far north and leans off-grid. Her birth timestamp
ships as the canonical example in her binary's help text: the engine's first test subject
was its author. She is currently open to Rust contracting and public-interest funding for
the stack.

GitHub: https://github.com/nutypebuddha · Flagship: https://github.com/nutypebuddha/L

---

## First-person short (~70 words — grant forms that ask "about you")

I'm Ashley (NutypeBuddha). I build L.ai, an offline-first verification stack for AI: a
deterministic proof engine, a per-token gate for LLM output, an MCP bridge, and a relational
reasoning layer — one binary that answers only what it can prove. I develop the whole thing
on an Android phone, license it Apache 2.0, and keep the bug ledger public. Verify, don't
trust — including me.
