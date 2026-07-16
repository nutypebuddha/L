# Laverna `verify` API & proof-object spec

Part of the "Laverna as an LLM verification substrate" roadmap (see
`laverna-roadmap-guide.md` §2.2 / §2.6). This document is the contract that the
`solve --proof-out` / `verify` pair implements. It is intentionally small: the
goal is a *proof-carrying* object an independent checker can validate without
re-running the original solver.

## 1. Trust model

The producer (`solve`) and the checker (`verify`) share as little code as is
practical. `verify` does not trust the recorded descent scores, tokens, or
digest — it **recomputes** the descent from the recorded `query` against the
embedded corpus and compares the canonical payload byte-for-byte. The recorded
claim is treated as untrusted input.

Two independent checks gate a successful verdict:

1. **Digest integrity** — does the recorded `digest.value` equal
   `SHA-256(canonical(recorded_payload))`? Catches a tampered digest or a
   hand-edited payload field that the forger forgot to re-hash.
2. **Recomputation** — does re-running the descent from the recorded `query`
   against the *currently embedded* corpus produce a byte-identical canonical
   payload? Catches a mutated corpus, a forged score token, or a tampered query.

Both must pass. Either alone is insufficient.

## 2. `verify` inputs

`laverna verify <path> [--format json|text]`

- `path` — filesystem path to a JSON proof object produced by `solve --proof-out`.
- `--format` — `text` (default, human-readable) or `json` (machine-readable
  verdict).

The proof object itself carries everything `verify` needs; there are no external
inputs. The recorded `query` is the only thing re-fed to the descent engine.

## 3. Proof-object schema

`build_proof_object` emits the following JSON. The **payload** (everything
except `digest` and `computed_at`) is the canonical value the digest is computed
over.

```jsonc
{
  "schema_version": "1.1.0",
  "query": "दशम भाव मे बुध",
  "laverna_version": "0.3.0",
  "corpus": {                       // Part 2.6: pin the knowledge base
    "version": "0.3.0",             //   CORPUS_VERSION (semver)
    "content_hash": "8a8aea9f230dfce8"  //   FNV-1a over embedded corpus
  },
  "descent": {
    "resolution_score": 0.0,        // 0..100
    "average_depth": 0.0,
    "layer_counts": { ... },
    "nand_completeness": 0.0,       // fraction of tokens settled to an absolute layer
    "tokens": [
      {
        "text": "बुध",
        "settled_layer": "Macro",
        "depth": 0,
        "domains": ["d0"],
        "formulas": 0,
        "entity": "...",
        "is_absolute": false,
        "confidence": 0.0
      }
    ]
  },
  "tanto_evaluations": [ { "token": "...", "expression": "...", "value": 0.0 } ],
  "dominant_domains": ["d0"],
  "intent": "...",
  "domain": "...",

  "digest": {                       // envelope — NOT part of the hashed payload
    "algorithm": "sha256",
    "value": "dfbc6cf5..."
  },
  "computed_at": "..."              // envelope — only present with --timestamp
}
```

### Field rules

- **Canonical serialization**: the digest is over
  `serde_json::to_string(payload)` (compact, key order as constructed). The
  payload is deterministic (no `HashMap` iteration — see determinism rule).
- **`computed_at` is an envelope field.** It is excluded from the digest so that
  proofs are byte-reproducible across runs and machines (T53). Off by default;
  enabled only with `solve --proof-out ... --timestamp`.
- **`corpus.content_hash` is in the payload.** A fork or CI rebuild that changes
  the corpus changes the payload and therefore the digest — corpus drift is
  caught automatically.

## 4. `verify` outputs

### `text`

```
  Query: "..."
  Corpus: v0.3.0 (content_hash 8a8aea9f230dfce8)
  ✓ Digest integrity (recorded matches payload)
  ✓ Recomputation (descent re-run matches recorded claim)

✓ Proof object VERIFIED (schema 1.1.0)
```

On failure, the diverging top-level payload fields are listed under the
recomputation line and the process exits non-zero (digest failure → exit 2;
recompute/verify failure → exit 1).

### `json`

```jsonc
{
  "verified": true,
  "query": "...",
  "digest_ok": true,
  "recompute_ok": true,
  "recorded_digest": "dfbc6cf5...",
  "recomputed_digest": "dfbc6cf5...",
  "schema_version": "1.1.0",
  "laverna_version": "0.3.0",
  "corpus_version": "0.3.0",
  "corpus_content_hash": "8a8aea9f230dfce8",
  "mismatched_fields": []
}
```

## 5. Error types

| Condition | Detected by | Exit |
|-----------|-------------|------|
| Proof file unreadable | `std::fs::read_to_string` | 2 |
| Proof not valid JSON | `serde_json::from_str` | 2 |
| Recorded digest ≠ hash(canonical payload) | digest integrity check | 1 |
| Recomputed payload ≠ recorded payload | recomputation check | 1 |

Typed refusals for *producing* a proof (vs. *checking* one) live in §2.3 of the
roadmap guide: `OutOfScope`, `Underspecified`, `TooComplex`, `NoTranslation`,
`MissingTimezone`. These are emitted by `solve`/`chart`/`route` when they cannot
construct a trustworthy claim in the first place.

## 6. Determinism contract

- Every field that touches a `HashMap` is sorted by a stable key before being
  serialized into the payload (T43 / T50). The payload is therefore byte-stable.
- The corpus hash uses FNV-1a (dependency-free, platform-stable) over the
  embedded corpus bytes, in `build.rs`. No crypto dependency is linked.
- Floating-point reduction order in `descent` is canonical; the proof pins
  numeric results rather than re-deriving them from raw inputs at verify time.
