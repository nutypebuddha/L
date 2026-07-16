# Known Issues

Public, committed, no euphemism. Documented bugs with scope and status.

---

### [T52] `strategize` exponential blowup on large budgets

**Status:** fixed-pending-release
**Affects:** `lai strategize --budget <N>` for N > ~20
**Does not affect:** `strategize --budget ≤15`, `optimize`, `build`, all other subcommands
**Repro:** `lai strategize --query "build a resilient distributed system" --budget 30`
**Detail:** Budget >20 caused exponential node expansion in the brute-force allocator (NODE_CAP=5M hit within seconds). Fixed by adding LP-relaxation upper bound for branch-and-bound pruning. Budget=30 now completes in ~0.6s. Test: `branch_and_bound_handles_large_budget`.

---

### [T53] `schema domain` template missing scoring table

**Status:** fixed-pending-release
**Affects:** `lai schema domain` output (template)
**Does not affect:** `schema optimize`, any hand-written domain profiles, `build`, `strategize`
**Repro:** `lai schema domain > /tmp/domain.toml && lai build --domain /tmp/domain.toml --datetime "2026-07-16" --tz "America/Chicago" --latitude 45.4 --longitude -92.9`
**Detail:** Template defined `score_cool` in graha_map but never declared a `[scoring.score_cool]` table, the `cool` item, or `score_cool` in `objective.maximize`. Running the template as-is failed immediately. Also fixed stale flags in header comment (`--lat`/`--lon`/`--datetime` → `--latitude`/`--longitude`/`--datetime --tz`).

---

### [MINOR] Corpus entity contains Chinese text in English description

**Status:** fixed-pending-release
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

**Status:** fixed-pending-release
**Affects:** `bridge` `/status` endpoint CID version field
**Does not affect:** `/validate`, `/fact`, `/health`, any actual validation logic
**Repro:** Run bridge without `CID_BINARY` set (stale path guarantees failure); `curl localhost:3000/status | jq .cid` returns `"v0.2.0 (binary found)"`
**Detail:** The `catch` block in `getCidVersion()` (line 66) returned the success-shaped string `'v0.2.0 (binary found)'`. Now returns `'v0.3.0 (binary NOT found)'`. Success branch updated to `'v0.3.0 (Tanto OK)'`.

---

### [T55] bridge `CID_BINARY` path stale post-monorepo refactor

**Status:** fixed-pending-release
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

**Status:** fixed-pending-release
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


