# Laverna-CLI: From Minivan to Formula 1

### A Complete Implementation Guide — Chart Precision → LLM Verification Substrate → Funding

**Scope:** This guide takes Laverna-CLI 0.3.0 from "works and surprised even you" to a two-pronged roadmap: (1) fixing the ephemeris/timezone foundation so the astrology engine is trustworthy, and (2) formalizing Laverna as a deterministic verification substrate for LLMs — the "Formula 1 car" the creative layer drives on. Everything here is grounded in the engineering research already done; this guide turns it into an ordered build plan.

---

## Part 0 — Why this order

The timezone bug you caught today is not a one-off — it's the small, concrete instance of the exact problem the LLM-synergy vision solves in general: **an unstated human assumption silently corrupting a deterministic calculation because nothing forced the input to be unambiguous.** Fixing it in the chart engine first is both the cheapest win available and a working case study you can point to when you pitch the bigger idea. Do the chart fixes before the LLM-substrate work — they're smaller, they're isolated to code you already understand, and they give you a concrete "before/after" proof point.

---

## Part 1 — Chart Engine: The Gyroscope Upgrade

### 1.1 Stop the bleeding: mandatory explicit timezone (do this first)

This is the fix for the exact bug you hit. It is a validation-boundary change, not a math change.

**Rule:** `chart` (and every other datetime-accepting command) must never accept an ambiguous local time. Require one of:
- A UTC timestamp (`--datetime-utc "1996-06-28T22:35:00Z"`), or
- A local datetime **plus** an explicit IANA timezone identifier (`--datetime "1996-06-28 17:35:00" --tz "America/Chicago"`)

Reject anything else with a typed, loud error — `MissingTimezone`, not a silent UTC assumption.

**Implementation:**
- Add `chrono-tz` or `jiff` (both bundle the IANA tz database) as a dependency.
- At the CLI parsing boundary — not deep in the chart logic — resolve local+tz → UTC instant. This means the rest of the chart pipeline only ever sees UTC, which is the correct architectural boundary (parse, don't validate downstream).
- Record the **resolved UTC instant and the applied offset** in the chart's output/proof object, so every chart is self-documenting about what time was actually used.
- For historical dates (pre-1970, and especially pre-1950s India-region dates if your corpus ever needs them), be aware the tz database itself flags uncertainty — surface that rather than implying false precision.

**Ticket to file:**
```
T-XX: chart/build/strategize commands accept ambiguous local time with no
timezone flag, causing silent wrong-UTC computation (confirmed: 1994-04-14
20:09 CDT cast as literal UTC produced Leo lagna instead of correct Libra
lagna — verified by re-running with manually-converted UTC time).

Fix: require --datetime-utc OR (--datetime + --tz), reject bare datetime
with MissingTimezone error. Add chrono-tz/jiff dependency. Record resolved
UTC + offset in proof output.

Severity: High — every existing chart cast without manual UTC conversion
is potentially wrong.
```

### 1.2 Ephemeris precision — what to use, what to avoid

**Do not use Swiss Ephemeris.** It's dual-licensed AGPL/commercial. Under AGPL, any network-accessible service using it (your `mcp` server mode counts) must release the entire application under AGPL, or you pay CHF 750–1550 per license. This is incompatible with staying Apache 2.0. This is a hard "no," not a tradeoff to weigh.

**Use instead:**
- **ERFA** (BSD-3-Clause, Apache-compatible) for IAU 2006 frame transforms — precession, nutation, sidereal time, Earth rotation. This is the community fork of IAU SOFA (SOFA itself has a restrictive "read-only" license — don't use SOFA directly, use ERFA).
  - Rust: the pure-Rust `erfa` crate (MPL-2.0) is closest to your no-FFI, small-binary needs, but it's incomplete — you'll likely need to port a few additional routines from the BSD-licensed C source yourself.
- **VSOP87** (or amplitude-ablated VSOP2013) for planetary positions — ~1 arcsecond accuracy, pure math, negligible data footprint. This is dramatically more than astrology needs (1″ ≈ 0.0003°, roughly 3000× finer than anything that matters for a sign/nakshatra cusp).
- **ELP-2000/82 or ELP/MPP02** for the Moon.
- Reference/candidate dependency: `celestial-ephemeris` (Apache-2.0 OR MIT, pure Rust, no FFI, amplitude-ablated VSOP2013 + ELP/MPP02) — worth evaluating directly as a dependency rather than reimplementing from scratch.

**Explicitly do not reach for JPL DE440/DE441** — sub-milliarcsecond precision, but 100+ MB of binary kernels. That's overkill for astrology and hostile to your offline/Termux/mobile footprint goals. Keep it as a future optional feature-flag if a use case ever demands it (eclipse timing, etc.) — not part of the core.

### 1.3 Move to 3D — the actual "gyroscope"

Your instinct was right: flat 2D degree arithmetic is the wrong representation. The correct chain (this is literally what ERFA implements):
```
Frame Bias → IAU 2006 Precession (P03) → IAU 2000A Nutation → Earth Rotation Angle / GST
```
Represent positions as 3D vectors; apply each stage as a rotation matrix (or quaternion) product; only project down to ecliptic longitude/latitude at the very end, right before you need a "rashi" or "nakshatra" label. This eliminates a whole class of cusp-boundary bugs that flat-degree math is prone to (wraparound errors, inconsistent epoch handling, accumulated rounding near 0°/360°).

### 1.4 Ayanamsa — the anchoring offset most implementations get subtly wrong

Lahiri ayanamsa = precession (which ERFA gives you) **plus a constant offset that anchors Spica (Chitra) at exactly 180° sidereal longitude.** Raw precession math alone gives you the wrong number — you need the anchoring constant too.
- Zero-ayanamsa epoch ≈ 285 CE.
- Approximate current value: ~24°06′–24°07′ in 2025–2026.
- **Distinguish official Lahiri (linear formula from the 285 CE epoch) from "True Chitrapaksha" (re-anchors Spica's real position each date)** — they diverge by 30″–60″, which is enough to flip a nakshatra pada or a sub-lord near a boundary. Pick one, document which, and **emit it explicitly in every chart's output** (you're already storing `ayanamsa` in the JSON — good — just make sure the method is unambiguous, not just the numeric value).

### 1.5 House cusps — know where Placidus breaks

If Laverna ever needs to support house systems beyond whole-sign: Placidus has no closed form (it trisects diurnal/nocturnal semi-arcs in *time*, requiring iterative convergence) and **it mathematically breaks above ~66° latitude** where degrees can be circumpolar. If you support Placidus, detect the high-latitude case and refuse loudly (`HighLatitudeHouseSystemUnsupported`) rather than emitting garbage cusps — or fall back to Whole Sign, which never breaks at any latitude and is traditional in Vedic work anyway.

### 1.6 Chart engine checklist

- [x] Add `chrono-tz`; require UTC (`--datetime-utc`) or local+IANA-tz (`--datetime`+`--tz`) at every datetime CLI boundary (`src/time.rs`)
- [x] Reject bare/ambiguous datetimes with a typed `MissingTimezone` error
- [x] Record resolved UTC + applied offset in chart/build output (text + JSON)
- [x] **No ERFA re-port needed**: traced the full lockfile — `astro` (MIT, pure Rust, ELP-2000/82 Moon + VSOP87 Sun/planets), `vsop87` (MIT/Apache-2.0), and `xalen-*` are already Apache-compatible and accurate. **No Swiss Ephemeris code or dependency exists anywhere** in the tree, so the license risk the guide warns about does not apply. `xalen-ayanamsa` already uses IAU 2006/P03 + IAU 2000B nutation + aberration (cross-validated vs Swiss SE_SIDM_TRUE_CITRA to 0.038″). Decided to keep these and document the decision (see `docs/` + commit) rather than discard correct, compatible code.
- [x] Refactor position math to a 3D vector layer at the boundary (`src/gyro/vec3.rs`): `Vec3` with `from_ecliptic_longitude`, `rotate_z` (ayanamsa as a rotation, not degree surgery), `rotate_axis` (Rodrigues, for the B→P→N→ERA chain), and `angular_separation_deg` (wraparound-safe aspect separation). Wired into the aspect matrix + synastry so 0°/360° wrap bugs can't occur. Existing planetary engines still supply the longitudes — only the framing is 3D.
- [x] Verify/document exact ayanamsa method and emit it: new `AyanamsaSystem` enum (`Lahiri` | `TrueChitra`) with a `--ayanamsa` flag on `chart`/`build` (+ MCP tools); both numeric value **and** method name emitted in every chart/proof output. Measured divergence for 1994-04-15: Lahiri 23.7808° vs True Chitrapaksha 23.7710° (~36″).
- [x] If supporting Placidus: `HouseSystem` enum + `HouseSystemError::PlacidusUnsupportedAtLatitude`, refused above ~66° latitude via `HouseSystem::validate_latitude` (wired into `with_location`). Whole Sign remains the only computed system and is valid at any latitude.
- [x] Re-cast known chart (1994-04-14 20:09 CDT) post-fix: now resolves to UTC `1994-04-15 01:09:00Z` → **Tula (♎ Libra)** lagna (was wrongly Simha/♌ when cast as literal UTC). Regression captured by the timezone fix test + manual diff.

---

## Part 2 — Laverna as an LLM Verification Substrate

### 2.1 The core architectural pattern (already validated by 2024–2026 research)

The pattern you're converging on independently is the same one the neurosymbolic-AI literature has landed on:
```
1. Autoformalization  — LLM translates natural language into a formal artifact
                         (a Tanto expression, a structured claim, a query)
2. Symbolic decision   — Laverna deterministically evaluates it: sat/unsat,
                         a computed value, or a routed strategy
3. Self-correction     — if Laverna rejects the input (parse error, out-of-
                         scope, underspecified), that typed error goes back
                         to the LLM for repair, not to the user as a crash
```
This is what Logic-LM, SatLM, and program-of-thought (PAL) systems do, and PAL-style code-execution verification alone closes a documented ~40-point accuracy gap over pure chain-of-thought reasoning on hard arithmetic benchmarks. You're not inventing the pattern — you're building the missing offline/deterministic/Rust implementation of it.

### 2.2 The interface contract — what a `verify` call actually needs

**Into the verify call:**
- The formal expression or claim (already-parsed Tanto, not raw NL — parsing/grounding happens upstream)
- Variable bindings / facts it depends on
- The declared domain/theory being asserted
- A declared scope (so out-of-scope detection has something to check against)

**Out of the proof object — what makes it trustworthy:**
- The canonicalized input expression
- The derivation steps or a certificate
- The corpus/version hash the derivation was made against (you already have `corpus hash` — wire this in everywhere)
- The deterministic result
- Enough information for a **small, independent checker** to re-validate without re-running the original solver — this is the proof-carrying-code principle: the consumer's job should be reduced from *proving* to *checking*.

**Design rule:** the `verify` checker should share as little code as possible with the thing that produced the proof. If the same code that generated a claim is the only thing that can check it, you haven't built a verifier — you've built a formatter.

### 2.3 Typed refusals — "fail loudly" as an API, not just a philosophy

Every rejection Laverna emits should be a specific, machine-readable type the calling LLM can react to programmatically, not a generic error string:
- `OutOfScope` — the query isn't something Laverna's corpus/formulas can address
- `Underspecified` — missing the bindings/facts needed to evaluate
- `TooComplex` — parseable but exceeds some bounded complexity limit
- `NoTranslation` — the LLM's attempted formalization didn't parse as valid Tanto
- `MissingTimezone` / other domain-specific input-validation failures (see Part 1)

This turns "fail loudly instead of hallucinate" from a slogan into something an LLM orchestration loop can actually branch on.

**Implemented.** `bankai::diagnostics::{RefusalKind, Refusal}` is the shared type; `DiagnosticReport::refuse()` records refusals (forcing `passed = false`) and `format_for_llm()` surfaces them. Wired across the CLI (main.rs):
- `solve`: empty query → `Underspecified`; all tokens unresolved at the Macro layer → `OutOfScope`; > `TOO_COMPLEX_TOKEN_LIMIT` (200) tokens → `TooComplex` (the deterministic descent caps the matrix to a bounded window, so it can't settle an unbounded query). Exit 1.
- `route`: empty → `Underspecified`; no token resolved to a corpus graha → `OutOfScope`; > 200 tokens → `TooComplex`. Exit 1.
- `validate`: empty → `Underspecified`; structural/parse failure → `NoTranslation`. Emitted inside the `DiagnosticReport` refusals list (JSON + text), exit 1.
- `chart` / `build`: missing/invalid timezone → `MissingTimezone` (structured `[REFUSAL MissingTimezone]` / JSON `refused:true`). Exit 2.

Every refusal carries a `fix_suggestion` so an orchestration loop can self-correct. Text form: `[REFUSAL <Kind>] <reason>` + `fix:`; JSON form: `{refused:true, kind, reason, fix_suggestion}`.

### 2.4 Determinism engineering — lock this down before you ship the pitch

You already have the right instincts (T43's HashMap tie-breaking bug proves it). Formalize it into a standing rule:
- **No `HashMap` iteration in any output path.** Use `BTreeMap`, sort before iterating, or a fixed-seed hasher. This is a documented, recurring bug class (it's bitten `rustc` itself).
- **Floating point is non-associative** — evaluation order changes results across platforms. Either fix reduction order canonically, or better: lean into your "no invented scalars" ethos and use exact rational arithmetic wherever the domain allows it, reserving floats for genuinely continuous quantities only.
- **Hash everything that should be reproducible.** SHA-256 over canonical serialization of the corpus (you have this) and of proof objects (extend it here) — this is what lets a community fork or CI catch drift.

### 2.5 Positioning: where you actually have open ground

The output-validation market (Guardrails AI, NVIDIA NeMo Guardrails, LMQL) is Python, online-oriented, and schema/policy-validation — not deterministic proof-emitting reasoning. **Nobody occupies "offline-first, deterministic, MCP-native, Rust verification sidecar."** That's a real, defensible gap, not wishful positioning.

Two concrete moves:
1. **Become the reference verification MCP server.** The MCP ecosystem has thousands of servers but no canonical trusted math/logic verifier. `laverna mcp` already exists — the work here is documentation, registry listing (mcp.so, Smithery), and a rock-solid `verify` checker, not new engineering.
2. **Publish your proof-object format as a small standard**, explicitly positioned against DRAT/LFSC (the mature SAT/SMT certificate formats) but aimed at everyday numeric/logical claims rather than formal math proofs. This is a standards play a solo dev can genuinely originate.

### 2.6 LLM-substrate checklist

- [x] Document the `verify` API contract (inputs, proof object schema, error types) as a standalone spec — `docs/verify-api.md` (Part 2.2 contract + this checklist item).
- [x] Implement typed refusal errors consistently across `solve`, `validate`, `route`, `chart` (see §2.3 — `RefusalKind`/`Refusal` in `bankai::diagnostics`, wired in `main.rs`; all five kinds live, each with a `fix_suggestion`).
- [x] Wire `corpus hash` into every proof object, not just corpus tooling — `build_proof_payload` now embeds `corpus.version` + `corpus.content_hash` (FNV-1a over the embedded corpus from `build.rs`); `verify` surfaces both in text + JSON. The corpus hash is inside the hashed payload, so a corpus drift fails verification automatically.
- [x] Audit for remaining non-deterministic collection types outside the already-fixed T43 case — done (Part 2.4): `entities.list`, `formulas.all`, `search`, and the registry's `by_domain`/`by_output`/single-token `search` are all uniformly id-sorted.
- [x] Write a short "fail loudly instead of hallucinate" manifesto with a concrete before/after (the timezone bug is a perfect example — LLM says "here's your chart," Laverna refuses instead of guessing) — `docs/manifesto.md`.
- [x] Get `laverna mcp` listed in at least one MCP registry (mcp.so / Smithery) — **submission assets ready**: `smithery.yaml` added at repo root + `docs/mcp-registry.md` with the mcp.so JSON config, Smithery YAML, and a card blurb. The `verify`/reasoning toolset is ready; the remaining action is pasting the assets into the registry web forms (no engineering left).
- [x] Draft the proof-object mini-spec as a public doc (even a README section) positioned against DRAT/LFSC — `docs/verify-api.md`.

---

## Part 3 — Funding: NLnet NGI Zero

This is a real, near-term-actionable path, not a maybe.
- **Program:** NGI Zero Commons Fund, administered by NLnet.
- **Amount:** individuals can request up to €50,000 as a first-time applicant; rolling deadlines (the 1st of even-numbered months).
- **Eligibility:** individuals qualify; Apache 2.0 is a recognized FOSS license.
- **Precedent:** NLnet has already funded "Provability Fabric" — described as open-source infrastructure "for making AI and software systems trustworthy through evidence that can be independently verified," integrating "formal verification, runtime security, and end-to-end audit trails." That is close enough to Laverna's pitch that the application essentially writes itself.

**Application framing:** don't pitch the astrology engine as the headline — pitch Laverna as *auditable, offline, deterministic verification infrastructure for trustworthy AI*, with the chart engine as one concrete proof-of-capability domain among the 528 formulas. Lead with:
- The proof-object / independent-checker architecture (Part 2.2)
- The determinism guarantees (Part 2.4) — content-addressed corpus, no invented scalars
- The offline-first / edge-deployment angle (Termux/Android as a real demonstrated environment, not a hypothetical)

**Order of operations for the application:**
1. Finish the timezone fix (Part 1.1) and the `verify` spec doc (Part 2.6) first — you want a concrete, working proof-carrying example to point to, not just a description.
2. Write the manifesto (Part 2.6) — this becomes the philosophical core of the grant narrative almost verbatim.
3. Apply citing Provability Fabric as the closest funded precedent.

**Status:** draft written — `docs/nlnet-application.md`. All three prerequisites
(Part 1.1 timezone fix, `docs/verify-api.md` proof-object spec, `docs/manifesto.md`
manifesto) are complete and the features they describe are implemented and
exported in the `v0.3.0` portable binary. Ready to submit on the next even-month
deadline.

---

## Suggested sequencing (everything above, in order)

1. **Timezone fix** (1.1) — smallest, isolated, immediately demoable
2. **Determinism audit** (2.4) — quick sweep, protects everything built after it
3. **ERFA + VSOP87 migration** (1.2, 1.3) — the real "gyroscope" work
4. **Ayanamsa verification** (1.4) — cheap correctness check once 3 is done
5. **`verify` spec + proof-object doc** (2.2, 2.6) — write this down before code drifts further from it
6. **Typed refusals across commands** (2.3) — mostly plumbing at this point
7. **Manifesto + MCP registry listing** (2.5, 2.6) — the public-facing pitch
8. **NLnet application** (Part 3) — submit once 1–7 give you real artifacts to cite

Everything after step 5 is positioning and packaging around an engine that, at that point, will actually deserve the confidence you're building toward.
