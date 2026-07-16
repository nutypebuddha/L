# L.ai — Brand & Normalization Guide

> Umbrella mark for the offline, WASM-native, fail-loud verification substrate.
> Code names (Laverna, CID, CID-Bridge) are internal; the public face is **L.ai**.

## The mark

```
        ___
       /   \     L  ·  a i
      |  ▲  |    (old-English L, witch-hat glitch)
      | ▛▀▀▛|
       \___/
```

The **L** is an old-English capital L (crossed/tailed descender) rendered as a
glitching cyberpunk witch-hat glyph: a sharp conical crown over a flickering
terminal block. It signals "the L that thinks" — logic under a hat, the
assistant that never guesses.

## Principle: name the function, not the thing

Every named artifact collapses to the **pure function** it performs. The brand
describes *what the function does*, not a mascot:

| Code name | Pure function | L.ai label |
|-----------|---------------|------------|
| Laverna (engine) | `deterministic_proof(input) -> verified_object` | L.ai · Proof |
| CID | `validate_token(stream) -> gated_verdict` (7 gates) | L.ai · Gate |
| CID-Bridge | `bridge(chatbot, [Gate, Proof]) -> merged_verdict` | L.ai · Bridge |
| Tanto (math) | `compute(expr) -> value` | L.ai · Compute |
| Companion | `assist(query) -> classified_receipt` | L.ai · Assist |

## Trademark

"Laverna" is conflicted (USPTO Mattel Reg. 3,303,422, Class 28 + an existing
open-source note app). The code names stay as *internal* identifiers; the
**commercial mark is L.ai** (with "Wintermore Housekeeping" reserved as the
monetization DBA). No USPTO filing of "Laverna".

## Usage

- README headers: `L.ai · <Function>` with the code name as a subtitle.
- All three repositories (Laverna, cid, cid-bridge) under `nutypebuddha` share
  this document verbatim.
- The verification contract is unchanged: pure functions only, no global state,
  deterministic output, fail loud.
