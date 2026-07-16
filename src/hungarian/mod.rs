//! Hungarian algorithm (Kuhn-Munkres) for assignment problems — T48.
//!
//! Hand-rolled for full determinism auditability. No external crate.
//! Solves the minimum-cost assignment problem on a square cost matrix in O(n³).
//!
//! Use case: LARP role↔player assignment, any bipartite matching where
//! each agent must be assigned to exactly one task.

/// Solve the assignment problem: given an n×n cost matrix, find the assignment
/// that minimizes total cost.
///
/// Returns a vector of (row, col) pairs — one assignment per row.
/// Ties are broken by lowest column index (deterministic).
pub fn hungarian(cost_matrix: &[Vec<f64>]) -> Result<Vec<(usize, usize)>, String> {
    let n = cost_matrix.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    for row in cost_matrix {
        if row.len() != n {
            return Err(format!(
                "Hungarian: cost matrix must be square, got {}×{}",
                n,
                row.len()
            ));
        }
    }

    // State-space guard
    const HUNGARIAN_N_CAP: usize = 5_000;
    if n > HUNGARIAN_N_CAP {
        return Err(format!(
            "Hungarian state-space guard: n={n} exceeds cap of {HUNGARIAN_N_CAP}"
        ));
    }

    // Step 1: Subtract row minima
    let mut cost = cost_matrix.to_vec();
    for row in &mut cost {
        let min = row.iter().cloned().fold(f64::INFINITY, f64::min);
        for val in row.iter_mut() {
            *val -= min;
        }
    }

    // Step 2: Subtract column minima
    #[allow(clippy::needless_range_loop)]
    for j in 0..n {
        let min = (0..n).map(|i| cost[i][j]).fold(f64::INFINITY, f64::min);
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            cost[i][j] -= min;
        }
    }

    // Step 3: Cover all zeros with minimum number of lines
    // Using the augmenting path method for the Hungarian algorithm
    let assignment = hungarian_inner(&cost, n)?;

    Ok(assignment)
}

/// Internal Hungarian implementation using the potential (label) method.
/// This is the O(n³) version due to Jonker-Volgenant.
fn hungarian_inner(cost: &[Vec<f64>], n: usize) -> Result<Vec<(usize, usize)>, String> {
    // u[i] = potential of row i, v[j] = potential of column j
    let mut u = vec![0.0f64; n + 1];
    let mut v = vec![0.0f64; n + 1];

    // match_of_col[j] = row matched to column j, or 0 if unmatched
    let mut match_of_col = vec![0usize; n + 1];

    // For each row, find the best augmenting path
    for i in 1..=n {
        // Start with row i, column 0 as a virtual column
        let mut assigned_col = 0usize;
        let mut min_slack = vec![f64::INFINITY; n + 1];
        let mut slack_col = vec![0usize; n + 1];
        let mut visited = vec![false; n + 1];

        // Virtual row 0 is always matched
        match_of_col[0] = i;

        // BFS/DFS to find augmenting path
        loop {
            visited[assigned_col] = true;
            let row_i = match_of_col[assigned_col];
            let mut delta = f64::INFINITY;
            let mut next_col = 0usize;

            for j in 1..=n {
                if visited[j] {
                    continue;
                }
                let reduced = cost[row_i - 1][j - 1] - u[row_i] - v[j];
                if reduced < min_slack[j] {
                    min_slack[j] = reduced;
                    slack_col[j] = assigned_col;
                }
                if min_slack[j] < delta {
                    delta = min_slack[j];
                    next_col = j;
                }
            }

            // Update potentials
            for k in 0..=n {
                if visited[k] {
                    u[match_of_col[k]] += delta;
                    v[k] -= delta;
                } else {
                    min_slack[k] -= delta;
                }
            }

            assigned_col = next_col;

            if match_of_col[assigned_col] == 0 {
                break;
            }
        }

        // Unwind augmenting path
        loop {
            let prev_col = slack_col[assigned_col];
            match_of_col[assigned_col] = match_of_col[prev_col];
            assigned_col = prev_col;
            if assigned_col == 0 {
                break;
            }
        }
    }

    // Extract assignment
    let mut result = Vec::with_capacity(n);
    #[allow(clippy::needless_range_loop)]
    for j in 1..=n {
        let row = match_of_col[j];
        result.push((row - 1, j - 1));
    }

    // Sort by row for deterministic output
    result.sort_by_key(|&(row, _)| row);

    Ok(result)
}

/// Compute the total cost of an assignment.
pub fn assignment_total_cost(cost_matrix: &[Vec<f64>], assignment: &[(usize, usize)]) -> f64 {
    assignment
        .iter()
        .map(|&(row, col)| cost_matrix[row][col])
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hungarian_simple() {
        let cost = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 1.0, 3.0],
            vec![3.0, 3.0, 1.0],
        ];

        let assignment = hungarian(&cost).unwrap();
        assert_eq!(assignment.len(), 3);

        // Optimal assignment: (0,0)=1, (1,1)=1, (2,2)=1 → total=3
        let total = assignment_total_cost(&cost, &assignment);
        assert!((total - 3.0).abs() < 1e-9);
    }

    #[test]
    fn hungarian_determinism() {
        let cost = vec![
            vec![10.0, 5.0, 13.0],
            vec![3.0, 7.0, 15.0],
            vec![12.0, 6.0, 9.0],
        ];

        let r1 = hungarian(&cost).unwrap();
        let r2 = hungarian(&cost).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn hungarian_1x1() {
        let cost = vec![vec![42.0]];
        let assignment = hungarian(&cost).unwrap();
        assert_eq!(assignment, vec![(0, 0)]);
        assert!((assignment_total_cost(&cost, &assignment) - 42.0).abs() < 1e-9);
    }

    #[test]
    fn hungarian_2x2() {
        let cost = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let assignment = hungarian(&cost).unwrap();
        let total = assignment_total_cost(&cost, &assignment);
        assert!((total - 2.0).abs() < 1e-9);
    }

    #[test]
    fn hungarian_non_square_error() {
        let cost = vec![vec![1.0, 2.0], vec![3.0]];
        assert!(hungarian(&cost).is_err());
    }

    #[test]
    fn hungarian_empty() {
        let cost: Vec<Vec<f64>> = Vec::new();
        let assignment = hungarian(&cost).unwrap();
        assert!(assignment.is_empty());
    }

    #[test]
    fn hungarian_state_space_guard() {
        let n = 5_001;
        let cost: Vec<Vec<f64>> = (0..n).map(|_| vec![0.0; n]).collect();
        let result = hungarian(&cost);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("state-space guard"));
    }

    #[test]
    fn hungarian_classic() {
        // Classic example from Wikipedia
        let cost = vec![
            vec![82.0, 83.0, 69.0, 92.0],
            vec![77.0, 37.0, 49.0, 92.0],
            vec![11.0, 69.0, 5.0, 86.0],
            vec![8.0, 9.0, 98.0, 23.0],
        ];
        let assignment = hungarian(&cost).unwrap();
        let total = assignment_total_cost(&cost, &assignment);
        // Optimal: (0,2)=69, (1,1)=37, (2,0)=11, (3,3)=23 → total=140
        assert!((total - 140.0).abs() < 1e-9);
    }
}
