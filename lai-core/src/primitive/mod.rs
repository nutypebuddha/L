//! # Primitive — NAND as the Bedrock Primitive
//!
//! All computation bottoms out to NAND gates. Every formula, every expression,
//! every gate compiles to a Directed Acyclic Graph (DAG) of NAND operations.

mod eval;
mod nand;

pub use eval::*;
pub use nand::*;

use std::collections::HashMap;

/// A node in the NAND computation DAG.
#[derive(Debug, Clone, PartialEq)]
pub enum NandNode {
    /// A named input variable
    Input(String),
    /// A constant literal value
    Constant(f64),
    /// A NAND gate combining two child nodes
    Nand { a: usize, b: usize },
}

/// A Directed Acyclic Graph of NAND operations.
#[derive(Debug, Clone)]
pub struct NandDag {
    nodes: Vec<NandNode>,
}

impl Default for NandDag {
    fn default() -> Self {
        Self::new()
    }
}

impl NandDag {
    /// Create a new empty NAND DAG.
    pub fn new() -> Self {
        NandDag { nodes: Vec::new() }
    }

    /// Add an input variable node, returning its index.
    pub fn add_input(&mut self, name: &str) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NandNode::Input(name.to_string()));
        idx
    }

    /// Add a constant value node, returning its index.
    pub fn add_constant(&mut self, value: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NandNode::Constant(value));
        idx
    }

    /// Add a NAND gate combining two existing nodes, returning its index.
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    pub fn add_nand(&mut self, a: usize, b: usize) -> usize {
        assert!(
            a < self.nodes.len(),
            "NandDag: node index {a} out of bounds"
        );
        assert!(
            b < self.nodes.len(),
            "NandDag: node index {b} out of bounds"
        );
        let idx = self.nodes.len();
        self.nodes.push(NandNode::Nand { a, b });
        idx
    }

    /// Derive `not(a)` from `nand(a, a)`.
    pub fn add_not(&mut self, a: usize) -> usize {
        self.add_nand(a, a)
    }

    /// Derive `and(a, b)` from `not(nand(a, b))`.
    pub fn add_and(&mut self, a: usize, b: usize) -> usize {
        let n = self.add_nand(a, b);
        self.add_not(n)
    }

    /// Derive `or(a, b)` from `nand(not(a), not(b))`.
    pub fn add_or(&mut self, a: usize, b: usize) -> usize {
        let na = self.add_not(a);
        let nb = self.add_not(b);
        self.add_nand(na, nb)
    }

    /// Derive `nor(a, b)` from `not(or(a, b))`.
    pub fn add_nor(&mut self, a: usize, b: usize) -> usize {
        let o = self.add_or(a, b);
        self.add_not(o)
    }

    /// Derive `xor(a, b)` from `or(and(a, not(b)), and(not(a), b))`.
    pub fn add_xor(&mut self, a: usize, b: usize) -> usize {
        let nb = self.add_not(b);
        let anb = self.add_and(a, nb);
        let na = self.add_not(a);
        let nab = self.add_and(na, b);
        self.add_or(anb, nab)
    }

    /// Derive `xnor(a, b)` from `not(xor(a, b))`.
    pub fn add_xnor(&mut self, a: usize, b: usize) -> usize {
        let x = self.add_xor(a, b);
        self.add_not(x)
    }

    /// Derive `implies(a, b)` from `or(not(a), b)`.
    pub fn add_implies(&mut self, a: usize, b: usize) -> usize {
        let na = self.add_not(a);
        self.add_or(na, b)
    }

    /// Evaluate the DAG with the given input bindings.
    pub fn evaluate(&self, inputs: &HashMap<String, f64>) -> Option<f64> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut values: Vec<f64> = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let v = match node {
                NandNode::Input(name) => inputs.get(name).copied()?,
                NandNode::Constant(c) => *c,
                NandNode::Nand { a, b } => {
                    let va = values[*a];
                    let vb = values[*b];
                    nand(va, vb)
                }
            };
            values.push(v);
        }
        values.last().copied()
    }

    /// Number of NAND gates in the DAG.
    pub fn nand_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| matches!(n, NandNode::Nand { .. }))
            .count()
    }

    /// Total number of nodes in the DAG.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the DAG is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the index of the last (output) node.
    pub fn output_index(&self) -> Option<usize> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes.len() - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nand_basic() {
        assert!((nand(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((nand(0.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((nand(1.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((nand(1.0, 1.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_not_derived() {
        assert!((not(0.0) - 1.0).abs() < 1e-12);
        assert!((not(1.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_all_gates_truth_tables() {
        // AND
        assert!((and(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((and(0.0, 1.0) - 0.0).abs() < 1e-12);
        assert!((and(1.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((and(1.0, 1.0) - 1.0).abs() < 1e-12);

        // OR
        assert!((or(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((or(0.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((or(1.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((or(1.0, 1.0) - 1.0).abs() < 1e-12);

        // NOR
        assert!((nor(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((nor(0.0, 1.0) - 0.0).abs() < 1e-12);
        assert!((nor(1.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((nor(1.0, 1.0) - 0.0).abs() < 1e-12);

        // XOR
        assert!((xor(0.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((xor(0.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((xor(1.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((xor(1.0, 1.0) - 0.0).abs() < 1e-12);

        // XNOR
        assert!((xnor(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((xnor(0.0, 1.0) - 0.0).abs() < 1e-12);
        assert!((xnor(1.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((xnor(1.0, 1.0) - 1.0).abs() < 1e-12);

        // Implies
        assert!((implies(0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((implies(0.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((implies(1.0, 0.0) - 0.0).abs() < 1e-12);
        assert!((implies(1.0, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_nand_dag_simple() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        let _out = dag.add_nand(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 1.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 0.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 0.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_nand_dag_not() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let _out = dag.add_not(a);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 0.0).abs() < 1e-12);

        inputs.insert("a".to_string(), 0.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_nand_dag_xor() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        let _out = dag.add_xor(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 0.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 1.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 1.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_nand_dag_implies() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        let _out = dag.add_implies(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 0.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 0.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 1.0);
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_nand_dag_constants() {
        let mut dag = NandDag::new();
        let a = dag.add_constant(1.0);
        let b = dag.add_constant(1.0);
        dag.add_nand(a, b);

        let inputs = HashMap::new();
        let result = dag.evaluate(&inputs).unwrap();
        assert!((result - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_nand_dag_empty() {
        let dag = NandDag::new();
        assert!(dag.is_empty());
        let mut inputs = HashMap::new();
        inputs.insert("x".to_string(), 1.0);
        assert!(dag.evaluate(&inputs).is_none());
    }

    #[test]
    fn test_continuous_truth_values() {
        let result = nand(0.3, 0.4);
        let expected = 1.0 - 0.3 * 0.4;
        assert!((result - expected).abs() < 1e-12);
    }
}
