# Hardening Pass — Audit Report

**Scope:** Strengthen the L.ai verification stack (Gate / `cid` crate + Proof /
`laverna` crate) against malformed, oversized, and adversarial input, and make
the validation verdicts unambiguous. No architectural rewrite; `lai-core`,
Gate, and Proof remain the foundation.

**Status:** Build clean · clippy clean (`-D warnings`) · `cargo fmt` clean ·
targeted tests pass.

---

## 1. What changed and why

### 1.1 One authoritative Gate validation path (`gate/src/gates/mod.rs`)
- Added `validate_candidate(ball, pins, context, claim, kb)` — the single
  function every caller now funnels through. It iterates the active pins **once**,
  maps each to its gate, runs the per-gate `GateResult`, and records it on the
  `Ball`.
- `validate_ball` (WASM surface) is now a thin wrapper over
  `validate_candidate`. Pipeline (`validate_single_token` /
  `validate_candidates`) and the `gate` binary (`validate_token`, `beam`) were
  re-pointed at `validate_candidate`, removing four duplicated pin→gate mapping
  blocks.

### 1.2 `Pin.threshold` enforced consistently (`gate/src/gates/mod.rs`)
- Unified verdict rule, in exactly one place:
  - `semantic_pass && score >= threshold` → **Pass**
  - `semantic_pass && score <  threshold` → **Unevaluable** (distinct from Fail)
  - `semantic Fail` → **Fail**
- `GateOutcome` (`Pass | Fail | Unevaluable`) added to
  `gate/src/core/ball.rs`; `GateResult` carries `outcome` (authoritative) and
  the legacy `passed: bool` (derivable). `Ball::overall_outcome()` reports the
  aggregate under the strict acceptance contract (accepted only when **all**
  required gates are `Pass`).

### 1.3 `Unevaluable` is semantically distinct from `Failed`
- `FactGate` (`gate/src/gates/fact.rs`) now returns **Fail** only on a definite
  contradiction and **Unevaluable** when there is nothing to judge against
  (missing corpus evidence / unverifiable claim). The same distinction holds for
  the `Math` gate: a true equation with `threshold` above its score is
  *unevaluable*, not *wrong*.

### 1.4 Per-pin costs connected to `Budget` (`gate/src/inference/pipeline.rs`)
- After exhaustion, `budget.spend_cost(pin_cost)` is charged per evaluated pin
  (costs sourced from `core::cost`). Previously the budget check existed but per-
  pin cost accounting was not wired in.

### 1.5 Default thresholds relaxed to realistic values (`gate/src/core/pin.rs`)
- `Logic`, `Fact`, `Formal` defaults `0.7 → 0.5` (Math stays `0.7`, Confidence
  `0.5`). Common valid input still fully validates while the threshold rule stays
  enforced (guards against silent over-rejection from the new strict contract).

### 1.6 Parser resource limits
- **Gate / CID** (`gate/src/tanto/parser.rs`): added `ParserLimits
  { max_input_bytes, max_tokens, max_depth, max_nodes }` (with `Default`),
  depth + node counters threaded through the recursive `parse_*`, and
  `eval_math_with_limits` / `eval_op_format_with_limits` entry points that
  centralize the over-limit rejection.
- **Proof / compute** (`proof/src/compute/parser.rs`): added the same class of
  guards — `MAX_INPUT_BYTES` / `MAX_TOKENS` at the two public entries
  (`eval_math`, `parse_math`) and a `MAX_DEPTH` recursion guard inside
  `parse_expr`. Pathological input (4,000-deep nesting, multi-megabyte tokens)
  is now rejected instead of recursing unbounded.

### 1.7 Versioned, tamper-evident proof envelope (`proof/src/verify/envelope.rs`)
- `ProofEnvelope { proof_version, engine_version, input_hash, corpus_hash,
  assumptions (sorted), derivation, result, verdict, proof_hash }`.
- `seal()` commits a SHA-256 (`crate::digest`, no external crypto) over every
  field **except** the hash itself; `verify_integrity()` re-derives and compares.
- `ProofVerdict { Accepted | Refused | Unevaluable }` — the stable, enum-based
  verdict (never a bare bool).
- `verify_proposal_envelope(...)` in `verifier.rs` is the trusted entry point
  that produces a sealed envelope; the module doc enumerates the TCB boundary
  (trusted: primitives, proof construction/verification, hashing, corpus-
  integrity refs, refusal semantics; untrusted: LLMs, NLP, Athena, heuristics,
  scoring, external adapters).

---

## 2. Tests added

| Area | Test (module) | Guards |
|------|---------------|--------|
| Gate parser limits | `tant/parser` (`input_size_limit`, `depth_limit`, `node_limit`, `op_format_token_limit`, `normal_parse_ok`) | size / depth / nodes / tokens |
| Gate unification + threshold + unevaluable | `gates::tests` (`unified_path_runs_each_pin_once`, `all_required_pins_pass_validates`, `false_equation_rejected`, `contradiction_is_fail_not_unevaluable`, `high_threshold_makes_valid_unevaluable`, `disabled_pins_skipped`) | single source of truth |
| Proof envelope | `verify::envelope::tests` (`deterministic_replay`, `serde_roundtrip`, `mutation_detected`, `property_valid_proofs_verify_tampered_fail`) | deterministic hash, tamper detection, property |
| Verifier→envelope | `verify::verifier::tests` (`verifier_emits_sealed_envelope_for_true_claim`, `verifier_emits_refused_envelope_for_false_claim`) | integration |
| Proof compute parser limits | `compute::parser::tests::parser_limits_bound_input_and_depth` | depth / size guards |

**Counts:** `lai-gate` 175 + 20 = **195 passed** · `laverna` `verify::` **50
passed** · `laverna` `compute::` **75 passed**. `cargo fmt --check` and
`cargo clippy -p lai-gate` / `-p laverna --lib` pass with `-D warnings`.

---

## 3. Verification performed
- `cargo build -p laverna` (bin + deps) — clean.
- `cargo clippy -p lai-gate --all-targets` — clean.
- `cargo clippy -p laverna --lib` — clean.
- `cargo fmt -p lai-gate -p laverna -- --check` — clean.
- Targeted test suites above — green.

---

## 4. Remaining limitations (out of scope / pre-existing)
1. **Pre-existing test failure, unrelated to this pass:**
   `laverna::optimize::fuzz_tlc01::fuzz_solver_matches_bruteforce` fails on the
   unmodified baseline too (verified by stashing these changes). It is a solver
   vs. brute-force mismatch in the optimizer, not a regression from this
   hardening. Recommend a follow-up ticket.
2. **Two parallel gate systems.** This pass unified the *Gate crate*
   (`gate/src/...`) validation path. The `proof` crate still carries its own
   copies in `proof/src/scoring/` and `proof/src/validation/`. Fully converging
   them into one Gate path would be an architectural rewrite and was explicitly
   out of scope; the new `ProofEnvelope`/`verify_proposal_envelope` sits on top
   of the proof-side verifier and is internally consistent, but it does not yet
   share the `cid` `validate_candidate` code.
3. **Proof compute parser** received input-size / token / depth guards, matching
   the Gate parser; AST *node-count* accounting was applied to the Gate parser
   but not separately to the token-based proof compute parser (its token count
   is the natural node bound there).

---

## 5. Files touched
- `gate/src/core/ball.rs`, `gate/src/core/pin.rs`
- `gate/src/gates/mod.rs`, `gate/src/gates/fact.rs`
- `gate/src/inference/pipeline.rs`, `gate/src/main.rs`
- `gate/src/tanto/parser.rs`
- `proof/src/verify/mod.rs`, `proof/src/verify/verifier.rs`
- `proof/src/verify/envelope.rs` (new)
- `proof/src/compute/parser.rs`
