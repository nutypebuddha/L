# Known Issues

Public, committed, no euphemism. Documented bugs with scope and status.

---

### [T52] `strategize` exponential blowup on large budgets

**Status:** released (v0.4.1)
**Affects:** `lai strategize --budget <N>` for N > ~20
**Does not affect:** `strategize --budget ≤15`, `optimize`, `build`, all other subcommands
**Repro:** `lai strategize --query "build a resilient distributed system" --budget 30`
**Detail:** Budget >20 caused exponential node expansion in the brute-force allocator (NODE_CAP=5M hit within seconds). Fixed by adding LP-relaxation upper bound for branch-and-bound pruning. Budget=30 now completes in ~0.6s. Test: `branch_and_bound_handles_large_budget`.

---

### [T53] `schema domain` template missing scoring table

**Status:** released (v0.4.1)
**Affects:** `lai schema domain` output (template)
**Does not affect:** `schema optimize`, any hand-written domain profiles, `build`, `strategize`
**Repro:** `lai schema domain > /tmp/domain.toml && lai build --domain /tmp/domain.toml --datetime "2026-07-16" --tz "America/Chicago" --latitude 45.4 --longitude -92.9`
**Detail:** Template defined `score_cool` in graha_map but never declared a `[scoring.score_cool]` table, the `cool` item, or `score_cool` in `objective.maximize`. Running the template as-is failed immediately. Also fixed stale flags in header comment (`--lat`/`--lon`/`--datetime` → `--latitude`/`--longitude`/`--datetime --tz`).

---

### [MINOR] Corpus entity contains Chinese text in English description

**Status:** released (v0.4.1)
**Affects:** `proof/entities/chakras.toml` (muladhara description)
**Does not affect:** any runtime behavior, parsing, or query resolution
**Repro:** `grep '安全感' proof/entities/chakras.toml`
**Detail:** Authoring slip: `安全感` (Chinese for "security") appeared in the English-language muladhara description. Replaced with "security".

---

### [ENV] `websearch` subcommand blocked in sandbox environments

**Status:** known, unscheduled
**Affects:** `lai websearch` when run behind TLS-inspecting proxies
**Does not affect:** `lai websearch` in normal network environments, all other subcommands
**Repro:** `lai websearch "GDP India"` (behind egress proxy)
**Detail:** HTTP client rejects proxy-intercepted TLS certificates. This is an environment-specific issue, not a code bug. The subcommand works correctly on standard networks. No fix planned — this is expected behavior for sandboxed builds.

---

### [T54] bridge `getCidVersion()` returns success string on failure

**Status:** released (v0.4.1)
**Affects:** `bridge` `/status` endpoint CID version field
**Does not affect:** `/validate`, `/fact`, `/health`, any actual validation logic
**Repro:** Run bridge without `CID_BINARY` set (stale path guarantees failure); `curl localhost:3000/status | jq .cid` returns `"v0.2.0 (binary found)"`
**Detail:** The `catch` block in `getCidVersion()` (line 66) returned the success-shaped string `'v0.2.0 (binary found)'`. Now returns `'v0.3.0 (binary NOT found)'`. Success branch updated to `'v0.3.0 (Tanto OK)'`.

---

### [T55] bridge `CID_BINARY` path stale post-monorepo refactor

**Status:** released (v0.4.1)
**Affects:** bridge shell-outs to CID engine (all validation via bridge)
**Does not affect:** gate CLI directly, proof, any Rust code
**Repro:** `node bridge/src/index.js` without `CID_BINARY` env; all `/validate` calls fail silently
**Detail:** Default path was `../../cid/target/release/cid` — `cid/` directory no longer exists (renamed to `gate/`). Fixed path through `lai-gate`, now `lai` (gate merged into unified binary). Every shell-out from bridge was failing silently, which is what triggered T54's false-positive catch. Additionally, the merge changed the CLI contract: `lai validate` is now Proof's Tanto-expression validator, not Gate's per-token validation. Gate's validate moved to `lai gate validate`. Bridge adapter updated accordingly.

---

### [T56] gate CLI: `--help`/`-h`/`help` silently blocks on stdin

**Status:** resolved (gate merged into unified `lai` binary)
**Affects:** standalone gate CLI (no longer exists)
**Does not affect:** `lai gate <subcommand>`, REPL mode, any current code paths
**Repro:** `lai-gate --help` (binary no longer exists; gate is lib-only)
**Detail:** This was a standalone gate binary issue. Gate is now lib-only, folded into the unified `lai` binary. `lai gate --help` works correctly via clap.

---

### [T57] gate README claims 13 MCP tools; actual count is 22

**Status:** released (v0.4.1)
**Affects:** `gate/README.md` documentation only
**Does not affect:** runtime behavior, MCP server, tool registration
**Repro:** `grep "13 tools" gate/README.md`
**Detail:** README claimed 13 MCP tools. Actual `list_tools()` returns 22 (8 original + 3 dynamic KB + 11 Tanto merged). Updated both references in README.

---

### [T58] `gate` subcommands have no JSON output mode; bridge `JSON.parse()` always fell through to fallback

**Status:** fixed
**Affects:** `bridge /validate` endpoint — could never distinguish correct from incorrect answers
**Does not affect:** `lai gate validate` CLI usage (text output), proof-side subcommands (already had `--format json`)
**Repro:** `POST /validate {"text":"2+2=4","context":"math"}` always returned `confidence: 0.5, passed: false`
**Detail:** `adapters/cid.js` did `JSON.parse(stdout)` on `gate validate`'s plaintext output (`Validated: ... Confidence: ...`). This always threw, landing in the catch block and returning the hardcoded fallback. Masked by T55 (wrong path) and the CLI contract bug — once both were fixed, the parse failure surfaced. Fixed by adding `--format json` to `gate validate`, `gate fix`, and `gate score` in `proof/src/main.rs` (matching the pattern used by proof-side subcommands), and wiring `adapters/cid.js` to pass `--format json`.

---

### [T59] Relational operators (<, >, <=, >=, !=) silently dropped or misread as `=`

**Status:** fixed
**Affects:** `lai validate` and `lai tanto eval` on relational expressions
**Does not affect:** `=` equations, plain arithmetic, `verify` (T52), `optimize`/`build`/`strategize`
**Repro:**
```
lai validate "9.11 < 9.9"    → passed: true (was true both directions before fix)
lai validate "9.11 > 9.9"    → passed: false (was: true)
lai validate "5 >= 3"        → passed: true (was: false — "Equation does not balance: 5 != 3")
```
**Detail:** Two independent gaps in `proof/src/compute/parser.rs` and `proof/src/validation/math_gate.rs`: (1) Tanto lexer had no relational token variants — `<`/`>` caused early termination, returning a truncated prefix as the full result. (2) `math_gate.rs`/`verifier.rs` split on the first `=` to check equation balance, which matched the `=` inside `<=`/`>=`/`!=`. Fixed by adding `Lt`/`Gt`/`Le`/`Ge`/`Ne` tokens with two-char lookahead, a comparison precedence level with epsilon tolerance, full-consumption checks in `eval_math`/`parse_math`, and routing relational expressions through Tanto before the `=` split in both `math_gate.rs` and `verify_arithmetic`. 625/625 tests pass, 0 regressions.

---

### [T56] assistant tests reference `termux`-gated `Intent` variants without gating

**Status:** released (v0.4.2)
**Affects:** `cargo test --workspace` (default features), `cargo fmt --check`, `cargo clippy --all-targets`
**Does not affect:** runtime behavior, the `termux` build, or any shipped binary
**Repro:** `cargo test --workspace` → `error[E0599]` ×3 in `assistant/src/intent/nlp.rs:909,922,930` (`SendMessage`, `Call`, `BatteryStatus` not found).
**Detail:** The `termux` feature gate was applied to the `Intent` enum variants and classifier branches (with a capability-honesty doc comment) but the `sms_basic`, `call_basic`, and `battery` unit tests were not gated to match. Consequence: all three dev-cycle gates (fmt, clippy, test) went red on default features — the last assistant batch landed without the gates running. Fix: added `#[cfg(feature = "termux")]` to the three tests and ran `cargo fmt --all`. Process fix: AGENTS.md now lists `assistant` in the workspace table and adds a `--features termux` CI leg so both configurations stay compile-clean. Also pinned the corpus size (214 entities / 528 formulas) with assert-eq tests so README stats cannot drift.

---

### [MINOR] Corpus size stat drift

**Status:** released (v0.4.2)
**Affects:** README "Embedded corpus" line (claimed 1,606+ facts)
**Does not affect:** runtime, parsing, or query resolution
**Repro:** `grep -c '\[\[entity\]\]' proof/entities/` → 214; `grep -c '\[\[formula\]\]' proof/formulas/` → 528.
**Detail:** The "1,606+ facts" figure was unsupported (the corpus is 214 entities + 528 formulas). Corrected the README and added `assert_eq!(len, 214)` / `assert_eq!(len, 528)` to the existing load-all tests in `proof/src/entity/mod.rs` and `proof/src/formula/registry.rs` so the documented counts are enforced.


