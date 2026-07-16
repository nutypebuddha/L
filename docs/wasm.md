# L.ai — WASM surface

Every L.ai function ships a WebAssembly module so it runs **offline, in-browser,
fail-loud** — the local-first thesis behind the NLnet applications.

## Modules

| Function | Crate | WASM crate | Size |
|----------|-------|------------|------|
| L.ai · Proof (Laverna) | `laverna` | `laverna-wasm` | ~590 KB |
| L.ai · Gate (CID) | `cid` | `cid-wasm` | ~615 KB |
| L.ai · Bridge (CID-Bridge) | `cid-bridge` (Node) | client-side fan-out to the two WASM modules | — |

## Shared loader pattern

Both `laverna-wasm` and `cid-wasm` follow the same build contract
(`build.sh` → `www/` with `wasm-bindgen --target web`):

- `init()` — no-op placeholder for symmetric loaders
- pure-function exports only; no global state
- deterministic output; content-addressed proof hash (`sha256`) on every result

A single `lai-wasm-loader` could wrap both, but each crate's `loader.js` already
demonstrates the contract independently.

## Offline-first

Because the verification logic is pure Rust compiled to WASM, a webpage can
validate LLM output (L.ai · Gate) and re-derive proofs (L.ai · Proof) entirely
client-side — no network round-trip, no server trust. This is the substrate the
ELFA application targets.
