# L.ai — Shared Verdict Contract

The unification rule (per `docs/brand.md`): every named artifact collapses to a
**pure function**. When L.ai functions compose, they exchange a single canonical
**verdict** shape. This document is the contract; it lives verbatim in all three
repositories (Laverna, CID, CID-Bridge).

## Gate taxonomy (shared enum)

The union of Laverna's `DiagnosticGate` and CID's `Gate` defines the canonical
gate set. Both crates re-export these names:

| Canonical gate | Laverna `DiagnosticGate` | CID `Gate` |
|----------------|--------------------------|------------|
| `Math` | ✓ | ✓ |
| `Logic` | ✓ | ✓ |
| `Formal` | ✓ | ✓ |
| `Fact` | — | ✓ |
| `Confidence` | ✓ | ✓ |
| `Fallacy` | ✓ | (gate) |
| `Structural` | ✓ | — |
| `Domain` | ✓ | — |

## Refusal taxonomy (shared enum)

Laverna's `RefusalKind` is the canonical typed-refusal vocabulary. CID's gate
failures SHOULD map onto these so a caller branching on a refusal sees one
enum across Proof + Gate:

```
RefusalKind ::=
    OutOfScope      # claim outside the model's domain
    Underspecified  # inputs missing / unresolvable
    TooComplex      # exceeds the deterministic solver's bounded state
    NoTranslation   # NLP could not map to a formula
    MissingTimezone # time-dependent claim without TZ (Laverna-specific)
```

## Verdict shape

```json
{
  "ok": true,
  "gate": "Math",
  "passed": true,
  "score": 0.98,
  "refusal": null,
  "hash": "<sha256 of the deterministic output>",
  "detail": ""
}
```

- `hash` is content-addressed (SHA-256) over the deterministic output, so any
   downstream consumer (L.ai · Bridge) can re-verify without trusting the producer.
- `refusal` is `null` on success, else one of `RefusalKind`.
- Outputs over unordered collections are sorted by a stable key before hashing
  (determinism rule from AGENTS.md / cid docs).

## Fact-source bridge (Proof → Gate)

Laverna's corpus is *formulas* (relationships), not scalar facts. It is exposed
to L.ai · Gate (CID) via `laverna corpus export --format cid-facts`, which emits
`cid_facts.json` (`schema: cid-facts/v1`): one entry per formula, each carrying a
SHA-256 of the formula text. CID can then cite a **re-verifiable** fact rather
than a static lookup — the "fail-loud, never-guess" contract extended across both.
