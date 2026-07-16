# Fail Loudly Instead of Hallucinate

*A manifesto for Laverna — auditable, offline, deterministic verification
infrastructure for trustworthy AI.*

## The problem in one sentence

An LLM that guesses is worse than an LLM that refuses — because the guess looks
authoritative and the refusal does not.

## The before: silence that looked like an answer

A user asks for a Vedic birth chart: *"April 14, 1994, 8:09 PM."* No timezone.
The old engine had two bad options, and it took the worse one:

- It could assume the input was UTC and silently compute. The result looked
  complete — lagna, bhavas, ayanamsa, the works. But 8:09 PM local in the US
  Central time zone is 1:09 AM **the next day** in UTC. Sidereal longitudes
  shift. The chart was for the wrong person entirely.
- Or it could throw a generic `error: invalid datetime` and leave the caller to
  guess what went wrong.

Both outcomes are failures of *trust*. The first invents a fact; the second
offers no machine-actionable signal an orchestration loop could recover from.

## The after: a typed refusal

Laverna now rejects bare local time with a specific, structured refusal:

```
[REFUSAL MissingTimezone] could not resolve chart datetime: missing timezone:
a chart datetime must be given as --datetime-utc (UTC instant) OR a local
--datetime together with an explicit --tz IANA timezone.
  fix: supply --datetime-utc <UTC ISO> or --datetime <local ISO> with --tz <IANA zone>
```

In JSON, the same refusal is `{ "refused": true, "kind": "MissingTimezone",
"reason": "...", "fix_suggestion": "..." }`. The calling LLM doesn't have to
parse prose — it branches on `kind` and retries with the corrected input.

Nothing is guessed. The computation either happens against an unambiguously
resolved UTC instant, or it does not happen.

## The principle

1. **No invented scalars.** If a value has to be assumed, the assumption is a
   refusal, not a default. A default timezone is a lie wearing a chart.
2. **Fail as an API, not a log line.** Every refusal is one of a fixed set of
   machine-readable kinds — `OutOfScope`, `Underspecified`, `TooComplex`,
   `NoTranslation`, `MissingTimezone` — each carrying a `fix_suggestion`. A
   slogan ("fail loudly") becomes something an LLM can act on.
3. **Be checkable, not just trustworthy.** Every proof object Laverna emits is
   content-addressed: the corpus version + content hash it was computed against
   is embedded in the payload, and a SHA-256 digest covers the whole. A
   verifier re-runs the descent from the recorded query and demands a
   byte-identical result. The consumer's job is reduced from *proving* to
   *checking*.
4. **Determinism is a correctness property, not a performance one.** No
   unordered collection is ever iterated in an output path; reduction order is
   canonical; the corpus is content-addressed so a drift fails verification
   instead of shipping a wrong answer.

## Why this matters for AI

LLMs are excellent generators and poor critics of their own output. The research
is unambiguous: an LLM given only its own reasoning cannot self-correct; it
needs an external signal. Laverna is that signal — offline, deterministic, and
small enough to run on a phone. It does not *replace* the model. It tells the
model, in a language the model's loop can branch on, exactly where it went
wrong and how to fix it.

The alternative — an LLM that confidently emits a chart for the wrong birth
moment, or a "verified" number it computed by guessing — is not a tool. It is a
flatterer. Laverna refuses to flatter.

**Fail loudly. Check independently. Trust nothing you can't reproduce.**
