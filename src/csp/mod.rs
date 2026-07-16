//! Small bounded CSP (Constraint Satisfaction Problem) solver — T49.
//!
//! Hand-rolled backtracking with constraint propagation.
//! Variable/value ordering is **explicitly fixed** (not left to the implementer's
//! whim) to guarantee determinism — same root cause as T43 if left implicit.
//!
//! Variable ordering: schema-declaration order (first variable tried first).
//! Value ordering: ascending numeric order (smallest value tried first).

/// A variable in the CSP with its domain of allowed values.
#[derive(Debug, Clone)]
pub struct CspVariable {
    pub id: String,
    pub domain: Vec<i64>,
}

/// A constraint: a predicate over a subset of variables.
/// Takes a slice of (variable_index, assigned_value) pairs and returns true
/// if the constraint is satisfied.
type CspPredicate = dyn Fn(&[(usize, i64)]) -> bool;

/// A constraint: a predicate over a subset of variables.
pub struct CspConstraint {
    pub name: String,
    /// Variable indices this constraint touches (must be sorted for determinism).
    pub scope: Vec<usize>,
    pub predicate: Box<CspPredicate>,
}

impl std::fmt::Debug for CspConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CspConstraint")
            .field("name", &self.name)
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

/// Solution to a CSP problem.
pub type CspSolution = Vec<(usize, i64)>; // (variable_index, value)

/// Solve the CSP using backtracking with forward checking.
///
/// Variables are tried in declaration order. Values are tried in ascending order.
/// This is the deterministic ordering that prevents T43-shaped bugs.
pub fn solve_csp(
    variables: &[CspVariable],
    constraints: &[CspConstraint],
) -> Result<Vec<CspSolution>, String> {
    if variables.is_empty() {
        return Ok(vec![Vec::new()]);
    }

    // State-space guard
    const CSP_STATE_CAP: usize = 1_000_000;
    let total_states = variables
        .iter()
        .try_fold(1usize, |acc, v| acc.checked_mul(v.domain.len()));
    match total_states {
        None | Some(0) => {
            return Err(
                "CSP state-space guard: state space too large (overflow or zero). \
                 Reduce variable domains or remove variables."
                    .to_string(),
            );
        }
        Some(n) if n > CSP_STATE_CAP => {
            return Err(format!(
                "CSP state-space guard: {n} states exceeds cap of {CSP_STATE_CAP}. \
                 Reduce variable domains or remove variables."
            ));
        }
        _ => {}
    }

    let mut solutions = Vec::new();
    let mut node_count = 0usize;
    let mut assignment: Vec<Option<i64>> = vec![None; variables.len()];

    backtrack(
        variables,
        constraints,
        &mut assignment,
        0,
        &mut solutions,
        &mut node_count,
    )?;

    Ok(solutions)
}

/// Find a single solution (faster for satisfiability checks).
pub fn solve_csp_one(
    variables: &[CspVariable],
    constraints: &[CspConstraint],
) -> Result<Option<CspSolution>, String> {
    if variables.is_empty() {
        return Ok(Some(Vec::new()));
    }

    const CSP_STATE_CAP: usize = 1_000_000;
    let total_states = variables
        .iter()
        .try_fold(1usize, |acc, v| acc.checked_mul(v.domain.len()));
    match total_states {
        None | Some(0) => {
            return Err(
                "CSP state-space guard: state space too large (overflow or zero).".to_string(),
            );
        }
        Some(n) if n > CSP_STATE_CAP => {
            return Err(format!(
                "CSP state-space guard: {n} states exceeds cap of {CSP_STATE_CAP}"
            ));
        }
        _ => {}
    }

    let mut solution = Vec::new();
    let mut node_count = 0usize;
    let mut assignment: Vec<Option<i64>> = vec![None; variables.len()];

    let found = backtrack_one(
        variables,
        constraints,
        &mut assignment,
        0,
        &mut solution,
        &mut node_count,
    )?;

    Ok(if found { Some(solution) } else { None })
}

/// Count solutions without storing them.
pub fn count_csp_solutions(
    variables: &[CspVariable],
    constraints: &[CspConstraint],
) -> Result<usize, String> {
    if variables.is_empty() {
        return Ok(1);
    }

    const CSP_STATE_CAP: usize = 1_000_000;
    let total_states = variables
        .iter()
        .try_fold(1usize, |acc, v| acc.checked_mul(v.domain.len()));
    match total_states {
        None | Some(0) => {
            return Err(
                "CSP state-space guard: state space too large (overflow or zero).".to_string(),
            );
        }
        Some(n) if n > CSP_STATE_CAP => {
            return Err(format!(
                "CSP state-space guard: {n} states exceeds cap of {CSP_STATE_CAP}"
            ));
        }
        _ => {}
    }

    let mut count = 0usize;
    let mut node_count = 0usize;
    let mut assignment: Vec<Option<i64>> = vec![None; variables.len()];

    backtrack_count(
        variables,
        constraints,
        &mut assignment,
        0,
        &mut count,
        &mut node_count,
    )?;

    Ok(count)
}

fn backtrack(
    variables: &[CspVariable],
    constraints: &[CspConstraint],
    assignment: &mut [Option<i64>],
    var_idx: usize,
    solutions: &mut Vec<CspSolution>,
    node_count: &mut usize,
) -> Result<(), String> {
    if var_idx == variables.len() {
        let solution: CspSolution = assignment
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|val| (i, val)))
            .collect();
        solutions.push(solution);
        return Ok(());
    }

    // Value ordering: ascending (deterministic)
    let mut sorted_domain = variables[var_idx].domain.clone();
    sorted_domain.sort();

    for value in sorted_domain {
        *node_count += 1;
        if *node_count > 1_000_000 {
            return Err("CSP state-space guard exceeded during search".to_string());
        }

        assignment[var_idx] = Some(value);

        if is_consistent(variables, constraints, assignment, var_idx) {
            backtrack(
                variables,
                constraints,
                assignment,
                var_idx + 1,
                solutions,
                node_count,
            )?;
        }
    }

    assignment[var_idx] = None;
    Ok(())
}

fn backtrack_one(
    variables: &[CspVariable],
    constraints: &[CspConstraint],
    assignment: &mut [Option<i64>],
    var_idx: usize,
    solution: &mut CspSolution,
    node_count: &mut usize,
) -> Result<bool, String> {
    if var_idx == variables.len() {
        *solution = assignment
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|val| (i, val)))
            .collect();
        return Ok(true);
    }

    let mut sorted_domain = variables[var_idx].domain.clone();
    sorted_domain.sort();

    for value in sorted_domain {
        *node_count += 1;
        if *node_count > 1_000_000 {
            return Err("CSP state-space guard exceeded during search".to_string());
        }

        assignment[var_idx] = Some(value);

        if is_consistent(variables, constraints, assignment, var_idx)
            && backtrack_one(
                variables,
                constraints,
                assignment,
                var_idx + 1,
                solution,
                node_count,
            )?
        {
            return Ok(true);
        }
    }

    assignment[var_idx] = None;
    Ok(false)
}

fn backtrack_count(
    variables: &[CspVariable],
    constraints: &[CspConstraint],
    assignment: &mut [Option<i64>],
    var_idx: usize,
    count: &mut usize,
    node_count: &mut usize,
) -> Result<(), String> {
    if var_idx == variables.len() {
        *count += 1;
        return Ok(());
    }

    let mut sorted_domain = variables[var_idx].domain.clone();
    sorted_domain.sort();

    for value in sorted_domain {
        *node_count += 1;
        if *node_count > 1_000_000 {
            return Err("CSP state-space guard exceeded during search".to_string());
        }

        assignment[var_idx] = Some(value);

        if is_consistent(variables, constraints, assignment, var_idx) {
            backtrack_count(
                variables,
                constraints,
                assignment,
                var_idx + 1,
                count,
                node_count,
            )?;
        }
    }

    assignment[var_idx] = None;
    Ok(())
}

/// Check if the current partial assignment is consistent with all constraints
/// whose scope is fully assigned.
fn is_consistent(
    _variables: &[CspVariable],
    constraints: &[CspConstraint],
    assignment: &[Option<i64>],
    var_idx: usize,
) -> bool {
    for constraint in constraints {
        // Only check constraints whose scope includes the just-assigned variable
        // and whose full scope is assigned
        if !constraint.scope.contains(&var_idx) {
            continue;
        }

        // Check if all variables in the scope are assigned
        let mut fully_assigned = true;
        let mut scope_values = Vec::new();
        for &idx in &constraint.scope {
            match assignment[idx] {
                Some(val) => scope_values.push((idx, val)),
                None => {
                    fully_assigned = false;
                    break;
                }
            }
        }

        if fully_assigned && !(constraint.predicate)(&scope_values) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csp_map_coloring() {
        // Classic map coloring: 3 regions, 3 colors, adjacent regions different
        let variables = vec![
            CspVariable {
                id: "WA".into(),
                domain: vec![0, 1, 2], // R, G, B
            },
            CspVariable {
                id: "NT".into(),
                domain: vec![0, 1, 2],
            },
            CspVariable {
                id: "SA".into(),
                domain: vec![0, 1, 2],
            },
        ];

        // Adjacent regions must have different colors
        let constraints = vec![
            CspConstraint {
                name: "WA≠NT".into(),
                scope: vec![0, 1],
                predicate: Box::new(|vals| vals[0].1 != vals[1].1),
            },
            CspConstraint {
                name: "WA≠SA".into(),
                scope: vec![0, 2],
                predicate: Box::new(|vals| vals[0].1 != vals[1].1),
            },
            CspConstraint {
                name: "NT≠SA".into(),
                scope: vec![1, 2],
                predicate: Box::new(|vals| vals[0].1 != vals[1].1),
            },
        ];

        let solutions = solve_csp(&variables, &constraints).unwrap();
        assert!(!solutions.is_empty());

        // Each solution should be valid
        for sol in &solutions {
            assert_eq!(sol.len(), 3);
            // Check all colors are different
            let colors: Vec<i64> = sol.iter().map(|&(_, c)| c).collect();
            assert!(colors[0] != colors[1]);
            assert!(colors[0] != colors[2]);
            assert!(colors[1] != colors[2]);
        }
    }

    #[test]
    fn csp_determinism() {
        let variables = vec![
            CspVariable {
                id: "x".into(),
                domain: vec![1, 2, 3],
            },
            CspVariable {
                id: "y".into(),
                domain: vec![1, 2, 3],
            },
        ];

        let constraints = vec![CspConstraint {
            name: "x≠y".into(),
            scope: vec![0, 1],
            predicate: Box::new(|vals| vals[0].1 != vals[1].1),
        }];

        let r1 = solve_csp(&variables, &constraints).unwrap();
        let r2 = solve_csp(&variables, &constraints).unwrap();
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn csp_single_solution() {
        let variables = vec![
            CspVariable {
                id: "x".into(),
                domain: vec![1],
            },
            CspVariable {
                id: "y".into(),
                domain: vec![2],
            },
        ];

        let solutions = solve_csp(&variables, &[]).unwrap();
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0], vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn csp_no_solution() {
        let variables = vec![
            CspVariable {
                id: "x".into(),
                domain: vec![1],
            },
            CspVariable {
                id: "y".into(),
                domain: vec![1],
            },
        ];

        let constraints = vec![CspConstraint {
            name: "x≠y".into(),
            scope: vec![0, 1],
            predicate: Box::new(|vals| vals[0].1 != vals[1].1),
        }];

        let solutions = solve_csp(&variables, &constraints).unwrap();
        assert!(solutions.is_empty());
    }

    #[test]
    fn csp_one_solution() {
        let variables = vec![
            CspVariable {
                id: "x".into(),
                domain: vec![1, 2, 3],
            },
            CspVariable {
                id: "y".into(),
                domain: vec![1, 2, 3],
            },
        ];

        let constraints = vec![CspConstraint {
            name: "x<y".into(),
            scope: vec![0, 1],
            predicate: Box::new(|vals| vals[0].1 < vals[1].1),
        }];

        let sol = solve_csp_one(&variables, &constraints).unwrap();
        assert!(sol.is_some());
        let sol = sol.unwrap();
        assert_eq!(sol.len(), 2);
        assert!(sol[0].1 < sol[1].1);
    }

    #[test]
    fn csp_count_solutions() {
        let variables = vec![
            CspVariable {
                id: "x".into(),
                domain: vec![1, 2, 3],
            },
            CspVariable {
                id: "y".into(),
                domain: vec![1, 2, 3],
            },
        ];

        let constraints = vec![CspConstraint {
            name: "x≠y".into(),
            scope: vec![0, 1],
            predicate: Box::new(|vals| vals[0].1 != vals[1].1),
        }];

        let count = count_csp_solutions(&variables, &constraints).unwrap();
        assert_eq!(count, 6); // 3*3 - 3 = 6 (all pairs minus same-color)
    }

    #[test]
    fn csp_empty() {
        let solutions = solve_csp(&[], &[]).unwrap();
        assert_eq!(solutions, vec![Vec::new()]);
    }

    #[test]
    fn csp_state_space_guard() {
        let variables: Vec<CspVariable> = (0..20)
            .map(|i| CspVariable {
                id: format!("v{i}"),
                domain: (0..10).collect(),
            })
            .collect();
        let result = solve_csp(&variables, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("state-space guard"));
    }
}
