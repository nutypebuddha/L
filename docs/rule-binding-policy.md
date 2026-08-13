# Rule-Binding Policy (the binder contract)

This is the stated contract for how `solve` connects the scalars it extracts
from a query to the formulas in the corpus, and how it chains them. It is the
sentence that makes the whole system legible in one line:

> **When a scalar could match more than one rule, the engine refuses rather than
> guess. A confident wrong answer is worse than no answer.**

This is the same rule as `route` returning `OutOfScope` and `validate` failing
loudly. The binder is the matching layer that applies it to arithmetic.

## Binding is by name, never by position

A scalar is bound to a formula input **only by matching the declared input name
in the query text**. There is no positional fallback path: positional binding is
the invented-scalar failure class — `ceil(8/13)*13` returned `13` as confidently
as `ceil(13/8)*8` returns `16`, and both look like answers. That is exactly the
hole the design exists to prevent, so position is not an available escape hatch.

Concretely:

- Each declared input name is matched against query tokens (exact word, or a
  word that starts/ends with the name). The match is **plural-tolerant**
  (downcased, a trailing `s` is ignored) so `bytes` matches `byte`, `sizes`
  matches `size`. A formula may also declare `input_aliases`
  (e.g. `size = ["extent", "length"]`, `mass = ["weight"]`) to accept synonyms.
  The scalar whose token is *nearest* by word distance is bound to it, under a
  unique one-scalar-per-input assignment (a minimum-distance injective match over
  the formula's inputs).
- A scalar may only bind to an input whose name token is within `MAX_BIND_DIST`
  (3) tokens of it. This rejects far "topic" mentions that have no real value.
- A scalar is **not** bound to an input when a *different* variable name sits
  between the input token and the number. That other variable owns the value, so
  binding it here would be a topic-word steal (e.g. "force … mass 5" must not let
  `force` capture the `5` that belongs to `mass`). This is the guard that stops
  the engine from hallucinating a scalar for a subject it merely named.
- Input names shorter than 3 characters (e.g. `a`, `b`) are **not** nameable
  from free text — they would match almost any word and manufacture a binding.
  Such inputs can only be filled from a derived output of an earlier rule, never
  from the query. An input whose name is short may still be anchored if it
  declares a `>=3`-character alias (e.g. `r = ["radius"]`).

## Every input and output carries a declared unit/type

Each input declares its unit (`input_types = { size = "bytes", align = "bytes" }`)
and each output declares `output_unit` (e.g. `"bytes"`). The binder infers a unit
for every extracted scalar from the query (a unit word embedded in the token like
`13-byte`, or in an adjacent token like `8 bytes`); a scalar with no unit word is
**untyped** and is accepted by any typed input.

Compatibility is **strict normalized identity**: `a` and `b` are compatible iff
either is untyped, or `normalize(a) == normalize(b)` (`bytes` == `byte`,
`m` == `meters`; `celsius` stays `celsius`). This is the NAND move applied to the
corpus:

- **Wrong-slot binding becomes unrepresentable.** A scalar of unit `celsius`
  cannot fill an input declared `"bytes"` — the pairing is not a candidate, so the
  formula does not bind. The silent-wrong-answer hole closes structurally, not by
  convention.
- **Conflict resolution gets a hard signal.** Two formulas may tie on keyword
  score; if only one's inputs typecheck against the extracted scalars, the other
  simply fails to bind and drops out of the candidate set. Most ties stop being
  ties.
- **Chaining is verified.** A derived output only feeds a consumer input whose
  declared unit matches (`bytes` → `bytes` composes; `bytes` → `angle` refuses).
  The two-hop chain can no longer silently feed a byte count into an angle.
- **The unit-normalization bug class can never recur**, because the unit is part
  of the value and a mismatch is refused rather than silently converted.

## Golden vectors — the corpus verifies itself

Every formula may declare known-correct input/output pairs (`[[formula.golden]]
inputs = { size = 13, align = 8 } output = 16`). The `corpus_golden_vectors`
integration test evaluates each formula's `expression` against its golden inputs
and asserts the declared output, on every CI run. This *executes* the `evidence`
field: a corpus of N formulas becomes N verified facts, not N claims.
- If an input has **no unambiguous name anchor** in the query and is not the
  output of an earlier rule in the chain, the binder **refuses** for that
  formula. It does not fall back to a default or to position.

## Ties at the top refuse

Among the formulas that *do* bind, the highest keyword score wins. If the top
two scores tie, the binder **refuses** (returns nothing) rather than picking one.
Same rule for the forward chain: if more than one rule could consume a derived
output, the highest score wins and a tie refuses the extra hop.

## Chaining (the rules engine)

After the primary rule fires, its **output variable** is fed forward into any
rule whose declared input has that same name. That is a two-hop application. The
chain continues as long as a uniquely-best consumer exists; it stops when no
rule consumes the latest output, or when the next hop is ambiguous.

- Each hop is independently name-bound and independently subject to the refusal
  rules above.
- A derived output never overwrites a scalar the user actually stated; it only
  fills an input the user did *not* state.

## What "refuse" means at the user boundary

When the binder refuses, `solve` prints `rule application: (no formula bound to
scals — refused/ambiguous)` (or omits the block). It does not emit a number. The
caller can then escalate — ask a clarifying question, surface `route`'s
`OutOfScope`, or fall back to the descent matrix — but the engine itself stays
silent rather than wrong.

## Determinism

Formula selection is by score; ties refuse, so the result does not depend on
HashMap iteration order. The injective name-assignment is a deterministic
minimum-distance match. The same query always yields the same binder result.
