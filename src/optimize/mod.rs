//! Universal stat/point-allocation optimizer (T41).
//!
//! Schemas are domain-agnostic TOML: a `[budget]`, a list of `[[items]]`
//! (continuous `attribute`s with per-point `cost`/`max_level`, or discrete
//! `perk`s gated by `requires`), an `[objective]` (what to `maximize` and with
//! what `weights`), and mandatory `[scoring.*]` terms. The solver builds the
//! Pareto frontier over the objective and selects via the weights — it never
//! invents a number, and fails loudly on any structural violation.

use std::collections::{BTreeMap, HashMap, HashSet};

#[cfg(feature = "milp")]
pub mod milp;

// ── Schema types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Schema {
    pub meta: Meta,
    pub budget: HashMap<String, f64>,
    pub items: Vec<Item>,
    pub objective: Objective,
    pub scoring: HashMap<String, ScoreTerm>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Meta {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub schema_version: u32,
    /// Solver shape — determines which algorithm is used.
    /// Valid values: "knapsack" (default), "milp", "assignment",
    /// "shortest_path", "mst", "max_flow", "interval_scheduling", "csp".
    #[serde(default)]
    pub shape: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Attribute,
    Perk,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Item {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: ItemKind,
    #[serde(default)]
    pub requires: Option<Vec<String>>,
    pub cost: HashMap<String, f64>,
    #[serde(default)]
    pub max_level: Option<u32>,
    #[serde(default)]
    pub effects: HashMap<String, f64>,
}

impl Item {
    pub fn is_attribute(&self) -> bool {
        matches!(self.kind, ItemKind::Attribute)
    }

    pub fn level_cap(&self) -> u32 {
        match self.kind {
            ItemKind::Attribute => self.max_level.unwrap_or(20),
            ItemKind::Perk => self.max_level.unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Objective {
    pub maximize: Vec<String>,
    #[serde(default)]
    pub weights: HashMap<String, f64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ScoreTerm {
    pub terms: HashMap<String, f64>,
}

// ── Result type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Allocation {
    pub levels: BTreeMap<String, u32>,
    pub stats: HashMap<String, f64>,
    pub scores: HashMap<String, f64>,
    pub objective: f64,
}

// ── Parse ───────────────────────────────────────────────────────────────────

pub fn parse_schema(toml_str: &str) -> Result<Schema, String> {
    toml::from_str(toml_str).map_err(|e| format!("schema parse error: {e}"))
}

// ── Prerequisite parsing ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CmpOp {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
}

impl CmpOp {
    fn check(self, a: f64, b: f64) -> bool {
        match self {
            CmpOp::Ge => a >= b,
            CmpOp::Gt => a > b,
            CmpOp::Le => a <= b,
            CmpOp::Lt => a < b,
            CmpOp::Eq => (a - b).abs() < 1e-9,
        }
    }
}

pub(crate) fn parse_prereq(s: &str) -> Result<(String, CmpOp, f64), String> {
    let s = s.trim();
    let (target, op, rest) = if let Some(i) = s.find(">=") {
        (&s[..i], CmpOp::Ge, &s[i + 2..])
    } else if let Some(i) = s.find("<=") {
        (&s[..i], CmpOp::Le, &s[i + 2..])
    } else if let Some(i) = s.find("==") {
        (&s[..i], CmpOp::Eq, &s[i + 2..])
    } else if let Some(i) = s.find('>') {
        (&s[..i], CmpOp::Gt, &s[i + 1..])
    } else if let Some(i) = s.find('<') {
        (&s[..i], CmpOp::Lt, &s[i + 1..])
    } else if let Some(i) = s.find('=') {
        (&s[..i], CmpOp::Eq, &s[i + 1..])
    } else {
        return Err(format!(
            "invalid requires clause '{s}' (expected e.g. id>=15)"
        ));
    };
    let target = target.trim().to_string();
    if target.is_empty() {
        return Err(format!("empty target in requires clause '{s}'"));
    }
    let threshold: f64 = rest
        .trim()
        .parse()
        .map_err(|_| format!("invalid threshold in requires clause '{s}'"))?;
    Ok((target, op, threshold))
}

// ── Validation (fail loudly) ──────────────────────────────────────────────────

/// Validate the schema. Returns `Err` on the first structural violation, with a
/// message naming the offending item/field.
pub fn validate_schema(schema: &Schema) -> Result<(), String> {
    for m in &schema.objective.maximize {
        if !schema.scoring.contains_key(m) {
            return Err(format!(
                "objective maximizes unknown score '{m}' (not defined in [scoring])"
            ));
        }
    }
    for w in schema.objective.weights.keys() {
        if !schema.scoring.contains_key(w) {
            return Err(format!("objective weight '{w}' is not a defined score"));
        }
    }
    let mut scored_keys: HashSet<String> = HashSet::new();
    for st in schema.scoring.values() {
        for k in st.terms.keys() {
            scored_keys.insert(k.clone());
        }
    }
    for item in &schema.items {
        for k in item.effects.keys() {
            if !scored_keys.contains(k) {
                return Err(format!(
                    "effect key '{}' (item '{}') is not declared in any [scoring.*] term",
                    k, item.id
                ));
            }
        }
    }
    let ids: HashSet<&str> = schema.items.iter().map(|i| i.id.as_str()).collect();
    for item in &schema.items {
        if let Some(reqs) = &item.requires {
            for r in reqs {
                let (target, _, _) = parse_prereq(r)?;
                if !ids.contains(target.as_str()) {
                    return Err(format!(
                        "requires target '{}' (item '{}') does not exist",
                        target, item.id
                    ));
                }
            }
        }
    }
    check_no_cycles(schema)?;
    Ok(())
}

fn check_no_cycles(schema: &Schema) -> Result<(), String> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for item in &schema.items {
        let mut targets = Vec::new();
        if let Some(reqs) = &item.requires {
            for r in reqs {
                let (target, _, _) = parse_prereq(r)?;
                targets.push(target);
            }
        }
        adj.insert(item.id.clone(), targets);
    }
    let mut color: HashMap<String, u8> = HashMap::new();
    let mut stack: Vec<String> = Vec::new();
    for item in &schema.items {
        if color.get(&item.id).copied().unwrap_or(0) == 0 {
            dfs_cycle(&item.id, &adj, &mut color, &mut stack)?;
        }
    }
    Ok(())
}

fn dfs_cycle(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    color: &mut HashMap<String, u8>,
    stack: &mut Vec<String>,
) -> Result<(), String> {
    color.insert(node.to_string(), 1);
    stack.push(node.to_string());
    if let Some(neighbors) = adj.get(node) {
        for n in neighbors {
            match color.get(n).copied().unwrap_or(0) {
                0 => dfs_cycle(n, adj, color, stack)?,
                1 => {
                    let start = stack.iter().position(|s| s == n).unwrap_or(0);
                    let path = stack[start..].join(" -> ");
                    return Err(format!("cycle in requires graph: {path} -> {n}"));
                }
                _ => {}
            }
        }
    }
    stack.pop();
    color.insert(node.to_string(), 2);
    Ok(())
}

// ── Evaluation helpers ────────────────────────────────────────────────────────

pub(crate) fn compute_stats(
    schema: &Schema,
    levels: &BTreeMap<String, u32>,
) -> HashMap<String, f64> {
    let mut stats: HashMap<String, f64> = HashMap::new();
    for item in &schema.items {
        let l = *levels.get(&item.id).unwrap_or(&0) as f64;
        if l == 0.0 {
            continue;
        }
        for (k, v) in &item.effects {
            *stats.entry(k.clone()).or_insert(0.0) += l * v;
        }
    }
    stats
}

pub(crate) fn compute_scores(
    schema: &Schema,
    stats: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let mut scores = HashMap::new();
    for (name, st) in &schema.scoring {
        let mut s = 0.0;
        for (k, coeff) in &st.terms {
            s += coeff * stats.get(k).copied().unwrap_or(0.0);
        }
        scores.insert(name.clone(), s);
    }
    scores
}

pub(crate) fn objective_value(schema: &Schema, scores: &HashMap<String, f64>) -> f64 {
    let mut total = 0.0;
    for name in &schema.objective.maximize {
        let w = schema.objective.weights.get(name).copied().unwrap_or(1.0);
        total += w * scores.get(name).copied().unwrap_or(0.0);
    }
    total
}

fn cost_of(schema: &Schema, levels: &BTreeMap<String, u32>) -> HashMap<String, f64> {
    let mut cost = HashMap::new();
    for item in &schema.items {
        let l = *levels.get(&item.id).unwrap_or(&0) as f64;
        if l == 0.0 {
            continue;
        }
        for (k, v) in &item.cost {
            *cost.entry(k.clone()).or_insert(0.0) += l * v;
        }
    }
    cost
}

fn within_budget(schema: &Schema, levels: &BTreeMap<String, u32>) -> bool {
    let cost = cost_of(schema, levels);
    for (res, avail) in &schema.budget {
        let used = cost.get(res).copied().unwrap_or(0.0);
        if used > avail + 1e-9 {
            return false;
        }
    }
    true
}

/// Maximum additional level of `item` that keeps the current partial assignment
/// within budget, for every budget resource. Lets the enumerator skip levels that
/// would provably exceed the budget — the feasible candidate set is unchanged,
/// but the search tree shrinks from a full `(cap+1)^K` product to the integer
/// compositions actually within budget (so 7 pillars × budget 20 stays far under
/// `NODE_CAP` instead of exploding).
fn level_ceiling(schema: &Schema, levels: &BTreeMap<String, u32>, item: &Item) -> u32 {
    let mut ceiling = item.level_cap();
    let cost = cost_of(schema, levels);
    for (res, avail) in &schema.budget {
        let used = cost.get(res).copied().unwrap_or(0.0);
        let per = item.cost.get(res).copied().unwrap_or(0.0);
        if per > 0.0 {
            let room = (avail - used) / per;
            let room = if room < 0.0 { 0.0 } else { room.floor() };
            ceiling = ceiling.min(room as u32);
        }
    }
    ceiling
}

fn prereqs_satisfied(schema: &Schema, levels: &BTreeMap<String, u32>) -> bool {
    for item in &schema.items {
        if *levels.get(&item.id).unwrap_or(&0) == 0 {
            continue;
        }
        if let Some(reqs) = &item.requires {
            for r in reqs {
                let (target, op, threshold) = match parse_prereq(r) {
                    Ok(p) => p,
                    Err(_) => return false,
                };
                let lvl = *levels.get(&target).unwrap_or(&0) as f64;
                if !op.check(lvl, threshold) {
                    return false;
                }
            }
        }
    }
    true
}

// ── Solver ───────────────────────────────────────────────────────────────────

/// State-space guard: maximum number of nodes the brute-force enumerator may
/// visit before aborting. Prevents runaway computation on oversized inputs.
/// 5M nodes ≈ a few seconds on modern hardware; well beyond any reasonable
/// domain-profile schema.
const NODE_CAP: usize = 5_000_000;

/// Solve the allocation problem and return the top-`top_k` distinct Pareto-optimal
/// allocations, ranked by the weighted objective (descending).
pub fn solve(schema: &Schema, top_k: usize) -> Result<Vec<Allocation>, String> {
    validate_schema(schema)?;
    let top_k = top_k.max(1);
    let attrs: Vec<&Item> = schema.items.iter().filter(|i| i.is_attribute()).collect();
    let perks: Vec<&Item> = schema.items.iter().filter(|i| !i.is_attribute()).collect();

    let mut candidates: Vec<Allocation> = Vec::new();
    let mut node_count = 0usize;
    let mut levels: BTreeMap<String, u32> = BTreeMap::new();
    enumerate_attributes(
        schema,
        &attrs,
        0,
        &mut levels,
        &perks,
        &mut candidates,
        &mut node_count,
    );

    if node_count > NODE_CAP {
        return Err(format!(
            "state-space guard exceeded: visited {node_count} nodes (cap {NODE_CAP}). \
             The schema has too many items/levels for brute-force enumeration. \
             Reduce max_level, remove items, or split into smaller sub-problems."
        ));
    }

    if candidates.is_empty() {
        return Err("no feasible allocation within budget and prerequisites".to_string());
    }

    let frontier = pareto_frontier(schema, &candidates);
    let mut ranked: Vec<Allocation> = frontier;
    ranked.sort_by(|a, b| {
        b.objective
            .partial_cmp(&a.objective)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| format!("{:?}", a.levels).cmp(&format!("{:?}", b.levels)))
    });

    let mut out: Vec<Allocation> = Vec::new();
    let mut seen: HashSet<Vec<(String, i64)>> = HashSet::new();
    for a in ranked {
        // Round objective values for dedup so near-equal frontier points collapse.
        let key: Vec<(String, i64)> = schema
            .objective
            .maximize
            .iter()
            .map(|m| {
                let v = a.scores.get(m).copied().unwrap_or(0.0);
                (m.clone(), (v * 1_000_000.0).round() as i64)
            })
            .collect();
        if seen.insert(key) {
            out.push(a);
            if out.len() >= top_k {
                break;
            }
        }
    }
    Ok(out)
}

fn enumerate_attributes(
    schema: &Schema,
    attrs: &[&Item],
    idx: usize,
    levels: &mut BTreeMap<String, u32>,
    perks: &[&Item],
    candidates: &mut Vec<Allocation>,
    node_count: &mut usize,
) {
    if idx == attrs.len() {
        enumerate_perks(schema, perks, 0, levels, candidates, node_count);
        return;
    }
    let item = attrs[idx];
    let cap = level_ceiling(schema, levels, item);
    for l in 0..=cap {
        *node_count += 1;
        if *node_count > NODE_CAP {
            return;
        }
        levels.insert(item.id.clone(), l);
        if within_budget(schema, levels) {
            enumerate_attributes(
                schema,
                attrs,
                idx + 1,
                levels,
                perks,
                candidates,
                node_count,
            );
        }
    }
    levels.insert(item.id.clone(), 0);
}

fn enumerate_perks(
    schema: &Schema,
    perks: &[&Item],
    idx: usize,
    levels: &mut BTreeMap<String, u32>,
    candidates: &mut Vec<Allocation>,
    node_count: &mut usize,
) {
    if idx == perks.len() {
        if within_budget(schema, levels) && prereqs_satisfied(schema, levels) {
            let stats = compute_stats(schema, levels);
            let scores = compute_scores(schema, &stats);
            let objective = objective_value(schema, &scores);
            candidates.push(Allocation {
                levels: levels.clone(),
                stats,
                scores,
                objective,
            });
        }
        return;
    }
    let item = perks[idx];
    levels.insert(item.id.clone(), 0);
    enumerate_perks(schema, perks, idx + 1, levels, candidates, node_count);
    *node_count += 1;
    if *node_count <= NODE_CAP {
        levels.insert(item.id.clone(), 1);
        if within_budget(schema, levels) {
            enumerate_perks(schema, perks, idx + 1, levels, candidates, node_count);
        }
        levels.insert(item.id.clone(), 0);
    }
}

fn pareto_frontier(schema: &Schema, candidates: &[Allocation]) -> Vec<Allocation> {
    let objs = &schema.objective.maximize;
    if objs.is_empty() || candidates.is_empty() {
        return candidates.to_vec();
    }

    // Fast path: single objective — just find the maximum.
    if objs.len() == 1 {
        let metric = &objs[0];
        let best = candidates
            .iter()
            .max_by(|a, b| {
                let av = a.scores.get(metric).copied().unwrap_or(0.0);
                let bv = b.scores.get(metric).copied().unwrap_or(0.0);
                av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();
        return best.into_iter().collect();
    }

    // Multi-objective: sort by first objective descending for early termination.
    let first_metric = &objs[0];
    let mut indexed: Vec<(usize, &Allocation)> = candidates.iter().enumerate().collect();
    indexed.sort_by(|a, b| {
        let av = a.1.scores.get(first_metric).copied().unwrap_or(0.0);
        let bv = b.1.scores.get(first_metric).copied().unwrap_or(0.0);
        bv.partial_cmp(&av)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut frontier = Vec::new();
    for &(i, a) in &indexed {
        let a_vec: Vec<f64> = objs
            .iter()
            .map(|m| a.scores.get(m).copied().unwrap_or(0.0))
            .collect();
        let mut dominated = false;
        for &(j, b) in &indexed {
            if i == j {
                continue;
            }
            // Early termination: if b's first objective < a's first objective,
            // then b cannot dominate a (since we sorted descending).
            let b_first = b.scores.get(first_metric).copied().unwrap_or(0.0);
            if b_first < a_vec[0] - 1e-9 {
                break;
            }
            let mut all_ge = true;
            let mut some_gt = false;
            for (k, &a_val) in a_vec.iter().enumerate() {
                let b_val = b.scores.get(&objs[k]).copied().unwrap_or(0.0);
                if b_val < a_val - 1e-9 {
                    all_ge = false;
                    break;
                }
                if b_val > a_val + 1e-9 {
                    some_gt = true;
                }
            }
            if all_ge && some_gt {
                dominated = true;
                break;
            }
        }
        if !dominated {
            frontier.push(a.clone());
        }
    }
    frontier
}

// ── Explanation ────────────────────────────────────────────────────────────────

/// Human-readable trace of a chosen allocation: which items were taken, their
/// cost and effect contributions, and the resulting score breakdown.
pub fn explain(schema: &Schema, alloc: &Allocation) -> String {
    let mut out = String::new();
    out.push_str("allocation:\n");
    let mut items: Vec<&Item> = schema.items.iter().collect();
    items.sort_by(|a, b| a.id.cmp(&b.id));
    for item in items {
        let l = *alloc.levels.get(&item.id).unwrap_or(&0);
        if l == 0 {
            continue;
        }
        let mut sorted_cost: Vec<_> = item.cost.iter().collect();
        sorted_cost.sort_by_key(|(k, _)| k.as_str());
        let cost: Vec<(&String, f64)> = sorted_cost
            .iter()
            .map(|(k, v)| (*k, *v * l as f64))
            .collect();
        let mut sorted_effects: Vec<_> = item.effects.iter().collect();
        sorted_effects.sort_by_key(|(k, _)| k.as_str());
        let effects: Vec<(&String, f64)> = sorted_effects
            .iter()
            .map(|(k, v)| (*k, *v * l as f64))
            .collect();
        out.push_str(&format!(
            "  {} (lvl {}) cost={:?} effects={:?}\n",
            item.id, l, cost, effects
        ));
    }
    out.push_str("scores:\n");
    let mut sorted_scores: Vec<_> = alloc.scores.iter().collect();
    sorted_scores.sort_by_key(|(k, _)| k.as_str());
    for (name, val) in sorted_scores {
        out.push_str(&format!("  {} = {:.4}\n", name, val));
    }
    out.push_str(&format!("objective (weighted) = {:.4}\n", alloc.objective));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_schema() -> Schema {
        Schema {
            meta: Meta {
                domain: "test".into(),
                schema_version: 1,
                shape: None,
            },
            budget: HashMap::new(),
            items: Vec::new(),
            objective: Objective {
                maximize: vec!["x".into()],
                weights: HashMap::new(),
            },
            scoring: HashMap::new(),
        }
    }

    fn attr(id: &str, cost: f64, max: u32, effect: (&str, f64)) -> Item {
        let mut c = HashMap::new();
        c.insert("pts".into(), cost);
        let mut e = HashMap::new();
        e.insert(effect.0.into(), effect.1);
        Item {
            id: id.into(),
            kind: ItemKind::Attribute,
            requires: None,
            cost: c,
            max_level: Some(max),
            effects: e,
        }
    }

    #[test]
    fn validation_rejects_unscored_effect() {
        let mut s = base_schema();
        s.items.push(attr("a", 1.0, 5, ("z", 1.0)));
        s.scoring.insert(
            "x".into(),
            ScoreTerm {
                terms: HashMap::new(),
            },
        );
        s.budget.insert("pts".into(), 5.0);
        assert!(validate_schema(&s).is_err());
    }

    #[test]
    fn validation_rejects_missing_requires_target() {
        let mut s = base_schema();
        let mut item = attr("a", 1.0, 5, ("x", 1.0));
        item.requires = Some(vec!["ghost>=1".into()]);
        s.items.push(item);
        let mut t = HashMap::new();
        t.insert("x".into(), 1.0);
        s.scoring.insert("x".into(), ScoreTerm { terms: t });
        s.budget.insert("pts".into(), 5.0);
        assert!(validate_schema(&s).is_err());
    }

    #[test]
    fn validation_rejects_cycle() {
        let mut s = base_schema();
        let mut x = attr("x", 1.0, 5, ("x", 1.0));
        x.requires = Some(vec!["y>=1".into()]);
        let mut y = attr("y", 1.0, 5, ("x", 1.0));
        y.requires = Some(vec!["x>=1".into()]);
        s.items.push(x);
        s.items.push(y);
        let mut t = HashMap::new();
        t.insert("x".into(), 1.0);
        s.scoring.insert("x".into(), ScoreTerm { terms: t });
        s.budget.insert("pts".into(), 10.0);
        let err = validate_schema(&s).unwrap_err();
        assert!(err.contains("cycle"), "got: {err}");
    }

    #[test]
    fn solver_picks_highest_marginal_attribute() {
        let mut s = base_schema();
        s.items.push(attr("a", 1.0, 10, ("x", 2.0)));
        s.items.push(attr("b", 1.0, 10, ("x", 1.0)));
        let mut t = HashMap::new();
        t.insert("x".into(), 1.0);
        s.scoring.insert("x".into(), ScoreTerm { terms: t });
        s.budget.insert("pts".into(), 10.0);
        let sols = solve(&s, 1).unwrap();
        assert_eq!(sols[0].levels.get("a").copied().unwrap_or(0), 10);
        assert_eq!(sols[0].levels.get("b").copied().unwrap_or(0), 0);
    }

    #[test]
    fn weights_are_applied_t28() {
        let mut s = base_schema();
        s.objective = Objective {
            maximize: vec!["p".into(), "q".into()],
            weights: {
                let mut w = HashMap::new();
                w.insert("p".into(), 2.0);
                w.insert("q".into(), 1.0);
                w
            },
        };
        s.items.push(attr("a", 1.0, 10, ("p", 1.0)));
        s.items.push(attr("b", 1.0, 10, ("q", 1.0)));
        let mut sc = HashMap::new();
        let mut tp = HashMap::new();
        tp.insert("p".into(), 1.0);
        sc.insert("p".into(), ScoreTerm { terms: tp });
        let mut tq = HashMap::new();
        tq.insert("q".into(), 1.0);
        sc.insert("q".into(), ScoreTerm { terms: tq });
        s.scoring = sc;
        s.budget.insert("pts".into(), 10.0);
        let sols = solve(&s, 1).unwrap();
        assert_eq!(sols[0].levels.get("a").copied().unwrap_or(0), 10);
        assert_eq!(sols[0].levels.get("b").copied().unwrap_or(0), 0);
    }

    #[test]
    fn prerequisite_gates_perk() {
        let mut s = base_schema();
        s.items.push(attr("a", 1.0, 20, ("s", 1.0)));
        let perk = Item {
            id: "p".into(),
            kind: ItemKind::Perk,
            requires: Some(vec!["a>=15".into()]),
            cost: {
                let mut c = HashMap::new();
                c.insert("perk_pts".into(), 5.0);
                c
            },
            max_level: Some(1),
            effects: {
                let mut e = HashMap::new();
                e.insert("s".into(), 10.0);
                e
            },
        };
        s.items.push(perk);
        let mut t = HashMap::new();
        t.insert("s".into(), 1.0);
        s.scoring.insert("s".into(), ScoreTerm { terms: t });
        s.objective = Objective {
            maximize: vec!["s".into()],
            weights: HashMap::new(),
        };
        s.budget.insert("pts".into(), 20.0);
        s.budget.insert("perk_pts".into(), 5.0);
        let sols = solve(&s, 1).unwrap();
        assert_eq!(sols[0].levels.get("p").copied().unwrap_or(0), 1);
        assert!(sols[0].levels.get("a").copied().unwrap_or(0) >= 15);
    }

    #[test]
    fn top_k_returns_distinct_frontier_points() {
        let mut s = base_schema();
        s.objective = Objective {
            maximize: vec!["p".into(), "q".into()],
            weights: HashMap::new(),
        };
        s.items.push(attr("a", 1.0, 10, ("p", 1.0)));
        s.items.push(attr("b", 1.0, 10, ("q", 1.0)));
        let mut sc = HashMap::new();
        let mut tp = HashMap::new();
        tp.insert("p".into(), 1.0);
        sc.insert("p".into(), ScoreTerm { terms: tp });
        let mut tq = HashMap::new();
        tq.insert("q".into(), 1.0);
        sc.insert("q".into(), ScoreTerm { terms: tq });
        s.scoring = sc;
        s.budget.insert("pts".into(), 10.0);
        let sols = solve(&s, 3).unwrap();
        assert!(sols.len() >= 2);
    }

    #[test]
    fn parse_real_toml_example() {
        let toml = r#"
[meta]
domain = "cyberpunk2077"
schema_version = 1

[budget]
attribute_points = 22
perk_points = 0

[[items]]
id = "technical_ability"
type = "attribute"
cost = { attribute_points = 3 }
max_level = 20
effects = { cyberware_capacity = 1.0, tech_dmg_pct = 0.5 }

[[items]]
id = "license_to_chrome_t1"
type = "perk"
requires = ["technical_ability>=15"]
cost = { perk_points = 5 }
effects = { cyberware_stat_mod_pct = 10.0 }

[objective]
maximize = ["survivability_score", "control_score"]
weights = { survivability_score = 0.5, control_score = 0.5 }

[scoring.survivability_score]
terms = { cyberware_capacity = 1.0 }

[scoring.control_score]
terms = { cyberware_stat_mod_pct = 0.2, tech_dmg_pct = 0.1 }
"#;
        let schema = parse_schema(toml).unwrap();
        validate_schema(&schema).unwrap();
        let sols = solve(&schema, 1).unwrap();
        assert_eq!(
            sols[0]
                .levels
                .get("license_to_chrome_t1")
                .copied()
                .unwrap_or(0),
            0
        );
    }
}
