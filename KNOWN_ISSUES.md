# Known Issues

Public, committed, no euphemism. Documented bugs with scope and status.

---

### [T52] `strategize` exponential blowup on large budgets

**Status:** fixed-pending-release
**Affects:** `laverna strategize --budget <N>` for N > ~20
**Does not affect:** `strategize --budget ≤15`, `optimize`, `build`, all other subcommands
**Repro:** `laverna strategize --query "build a resilient distributed system" --budget 30`
**Detail:** Budget >20 caused exponential node expansion in the brute-force allocator (NODE_CAP=5M hit within seconds). Fixed by adding LP-relaxation upper bound for branch-and-bound pruning. Budget=30 now completes in ~0.6s. Test: `branch_and_bound_handles_large_budget`.

---

### [T53] `schema domain` template missing scoring table

**Status:** fixed-pending-release
**Affects:** `laverna schema domain` output (template)
**Does not affect:** `schema optimize`, any hand-written domain profiles, `build`, `strategize`
**Repro:** `laverna schema domain > /tmp/domain.toml && laverna build --domain /tmp/domain.toml --datetime "2026-07-16" --tz "America/Chicago" --latitude 45.4 --longitude -92.9`
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
**Affects:** `laverna websearch` when run behind TLS-inspecting proxies
**Does not affect:** `laverna websearch` in normal network environments, all other subcommands
**Repro:** `laverna websearch "GDP India"` (behind egress proxy)
**Detail:** HTTP client rejects proxy-intercepted TLS certificates. This is an environment-specific issue, not a code bug. The subcommand works correctly on standard networks. No fix planned — this is expected behavior for sandboxed builds.

---

### [COSMETIC] `route` output may vary across runs on identical input

**Status:** known, unscheduled
**Affects:** `laverna route --query <same-query>` (output ordering)
**Does not affect:** correctness of individual route suggestions, `solve`, `verify`
**Repro:** `laverna route --query "system design" | md5sum` (run twice, compare)
**Detail:** Route scoring uses internal `HashMap` iteration for weight aggregation. While individual scores are deterministic, final ranking tie-breaking may vary. Not a correctness issue — all suggested routes are valid. Sorted by domain index for stable output where possible.
