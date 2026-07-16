//! MILP (Mixed Integer Linear Programming) solver tier — T46.
//!
//! Uses `good_lp` with the `microlp` backend: pure Rust, no C toolchain,
//! single-threaded so no solver-level determinism risk. Only compiled with
//! the `milp` feature gate.
//!
//! This handles general linear/integer allocation problems that exceed what
//! the brute-force knapsack enumerator can handle. For bounded knapsack,
//! use the default `knapsack` shape instead.

use good_lp::{variable, variables, Solution, SolverModel};

use super::{Allocation, Schema};

/// Solve the schema as a MILP problem.
///
/// The approach: for each item with `max_level`, create an integer variable
/// `x_item` ∈ [0, max_level]. Budget constraints are linear: Σ cost[k][i] * x_i ≤ budget[k].
/// Prerequisites become direct constraints: x_A ≤ x_B.
/// The objective is a weighted sum of scores, where scores are linear in
/// the item levels (via effects × scoring terms).
///
/// Returns the single best allocation found by the solver, or an error.
pub fn solve_milp(schema: &Schema) -> Result<Allocation, String> {
    validate_milp_feasible(schema)?;

    let mut vars = variables! {};

    // One integer variable per item
    let mut vars_and_ids: Vec<(String, good_lp::Variable)> = Vec::new();
    for item in &schema.items {
        let cap = item.level_cap();
        let v = variable().integer().clamp(0, cap as i32);
        let var = vars.add(v);
        vars_and_ids.push((item.id.clone(), var));
    }

    // Build objective: weighted sum of scores, where score = Σ(effect × scoring_coeff)
    // Precompute linear coefficient of each item's variable in the objective.
    let mut objective = good_lp::Expression::from(0.0);
    for (item_id, var) in &vars_and_ids {
        let item = schema.items.iter().find(|i| i.id == *item_id).unwrap();
        let mut total_coeff = 0.0f64;

        for (score_name, score_term) in &schema.scoring {
            let weight = schema
                .objective
                .weights
                .get(score_name)
                .copied()
                .unwrap_or(0.0);
            if weight == 0.0 {
                continue;
            }
            for (stat, coeff) in &score_term.terms {
                let effect = item.effects.get(stat).copied().unwrap_or(0.0);
                total_coeff += weight * coeff * effect;
            }
        }

        if total_coeff != 0.0 {
            objective += *var * total_coeff;
        }
    }

    let mut problem = vars.maximise(objective).using(good_lp::default_solver);

    // Budget constraints
    for (resource, budget_limit) in &schema.budget {
        let mut total_cost = good_lp::Expression::from(0.0);
        for (item_id, var) in &vars_and_ids {
            let item = schema.items.iter().find(|i| i.id == *item_id).unwrap();
            let cost = item.cost.get(resource).copied().unwrap_or(0.0);
            if cost > 0.0 {
                total_cost += *var * cost;
            }
        }
        problem = problem.with(total_cost << *budget_limit);
    }

    // Prerequisite constraints: if item A requires item B >= threshold,
    // then x_A <= x_B (for threshold=1, the most common case)
    for (item_id_a, var_a) in &vars_and_ids {
        let item = schema.items.iter().find(|i| i.id == *item_id_a).unwrap();
        if let Some(reqs) = &item.requires {
            for req in reqs {
                if let Ok((target_id, _op, _threshold)) = super::parse_prereq(req) {
                    if let Some((_, var_b)) = vars_and_ids.iter().find(|(id, _)| *id == target_id) {
                        problem = problem.with(*var_a << *var_b);
                    }
                }
            }
        }
    }

    // Solve
    let solution = problem
        .solve()
        .map_err(|e| format!("MILP solver error: {e}"))?;

    // Extract variable values
    let mut levels = std::collections::BTreeMap::new();
    for (item_id, var) in &vars_and_ids {
        let value = solution.value(*var) as u32;
        levels.insert(item_id.clone(), value);
    }

    // Compute stats, scores, objective using existing helpers
    let stats = super::compute_stats(schema, &levels);
    let scores = super::compute_scores(schema, &stats);
    let objective = super::objective_value(schema, &scores);

    Ok(Allocation {
        levels,
        stats,
        scores,
        objective,
    })
}

/// State-space guard: if the problem has more than this many integer
/// variables, reject it (it's likely too large for the MILP tier to
/// handle in reasonable time).
const MILP_VAR_CAP: usize = 10_000;

fn validate_milp_feasible(schema: &Schema) -> Result<(), String> {
    if schema.items.len() > MILP_VAR_CAP {
        return Err(format!(
            "MILP state-space guard: {} items exceeds cap of {MILP_VAR_CAP}",
            schema.items.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::{self, Item, ItemKind, Meta, Objective, ScoreTerm};

    fn make_simple_schema() -> Schema {
        Schema {
            meta: Meta {
                domain: "test".into(),
                schema_version: 1,
                shape: Some("milp".into()),
            },
            budget: [("points".into(), 10.0)].into_iter().collect(),
            items: vec![
                Item {
                    id: "a".into(),
                    kind: ItemKind::Attribute,
                    requires: None,
                    cost: [("points".into(), 2.0)].into_iter().collect(),
                    max_level: Some(5),
                    effects: [("dps".into(), 3.0)].into_iter().collect(),
                },
                Item {
                    id: "b".into(),
                    kind: ItemKind::Attribute,
                    requires: None,
                    cost: [("points".into(), 3.0)].into_iter().collect(),
                    max_level: Some(3),
                    effects: [("dps".into(), 5.0)].into_iter().collect(),
                },
            ],
            objective: Objective {
                maximize: vec!["power".into()],
                weights: [("power".into(), 1.0)].into_iter().collect(),
            },
            scoring: [(
                "power".into(),
                ScoreTerm {
                    terms: [("dps".into(), 1.0)].into_iter().collect(),
                },
            )]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn milp_returns_better_or_equal_to_knapsack() {
        let schema = make_simple_schema();
        let milp_result = solve_milp(&schema).expect("MILP should solve");
        let mut knapsack_result = optimize::solve(&schema, 1).expect("knapsack should solve");
        let knapsack = knapsack_result.remove(0);

        // MILP should find at least as good a solution
        assert!(
            milp_result.objective >= knapsack.objective - 1e-6,
            "MILP objective {} < knapsack objective {}",
            milp_result.objective,
            knapsack.objective
        );
    }

    #[test]
    fn milp_determinism() {
        let schema = make_simple_schema();
        let r1 = solve_milp(&schema).unwrap();
        let r2 = solve_milp(&schema).unwrap();
        assert_eq!(r1.levels, r2.levels);
        assert!((r1.objective - r2.objective).abs() < 1e-9);
    }

    #[test]
    fn milp_respects_budget() {
        let schema = make_simple_schema();
        let result = solve_milp(&schema).unwrap();
        let total_cost: f64 = schema
            .items
            .iter()
            .map(|item| {
                let level = result.levels.get(&item.id).copied().unwrap_or(0);
                let cost_per = item.cost.get("points").copied().unwrap_or(0.0);
                cost_per * level as f64
            })
            .sum();
        assert!(
            total_cost <= 10.0 + 1e-6,
            "total cost {total_cost} exceeds budget 10.0"
        );
    }

    #[test]
    fn milp_state_space_guard() {
        let mut schema = make_simple_schema();
        // Add enough items to exceed the cap
        for i in 0..10_001 {
            schema.items.push(Item {
                id: format!("extra_{i}"),
                kind: ItemKind::Attribute,
                requires: None,
                cost: [("points".into(), 0.01)].into_iter().collect(),
                max_level: Some(1),
                effects: [("dps".into(), 0.001)].into_iter().collect(),
            });
        }
        let result = solve_milp(&schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("state-space guard"));
    }
}
