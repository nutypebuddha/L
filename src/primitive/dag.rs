use std::collections::HashMap;

/// A node in the NAND computation DAG.
#[derive(Debug, Clone, PartialEq)]
pub enum NandNode {
    /// A named input variable.
    Input(String),
    /// A constant literal value.
    Constant(f64),
    /// A NAND gate combining two child nodes.
    Nand { a: usize, b: usize },
}

/// A Directed Acyclic Graph of NAND operations.
///
/// Immutable once built. Evaluation propagates values through the graph
/// in topological order.
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
    pub fn new() -> Self {
        NandDag { nodes: Vec::new() }
    }

    pub fn add_input(&mut self, name: &str) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NandNode::Input(name.to_string()));
        idx
    }

    pub fn add_constant(&mut self, value: f64) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NandNode::Constant(value));
        idx
    }

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

    pub fn add_not(&mut self, a: usize) -> usize {
        self.add_nand(a, a)
    }

    pub fn add_and(&mut self, a: usize, b: usize) -> usize {
        let n = self.add_nand(a, b);
        self.add_not(n)
    }

    pub fn add_or(&mut self, a: usize, b: usize) -> usize {
        let na = self.add_not(a);
        let nb = self.add_not(b);
        self.add_nand(na, nb)
    }

    pub fn add_nor(&mut self, a: usize, b: usize) -> usize {
        let o = self.add_or(a, b);
        self.add_not(o)
    }

    pub fn add_xor(&mut self, a: usize, b: usize) -> usize {
        let nb = self.add_not(b);
        let anb = self.add_and(a, nb);
        let na = self.add_not(a);
        let nab = self.add_and(na, b);
        self.add_or(anb, nab)
    }

    pub fn add_xnor(&mut self, a: usize, b: usize) -> usize {
        let x = self.add_xor(a, b);
        self.add_not(x)
    }

    pub fn add_implies(&mut self, a: usize, b: usize) -> usize {
        let na = self.add_not(a);
        self.add_or(na, b)
    }

    /// Evaluate the DAG with the given input bindings.
    /// Returns the computed value of the last node.
    pub fn evaluate(&self, inputs: &HashMap<String, f64>) -> Option<f64> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut values: Vec<f64> = Vec::with_capacity(self.nodes.len());
        let mut resolved: HashMap<&str, f64> = HashMap::with_capacity(8);
        for node in &self.nodes {
            let v = match node {
                NandNode::Input(name) => {
                    if let Some(&v) = resolved.get(name.as_str()) {
                        v
                    } else {
                        let v = inputs.get(name.as_str()).copied()?;
                        resolved.insert(name.as_str(), v);
                        v
                    }
                }
                NandNode::Constant(c) => *c,
                NandNode::Nand { a, b } => {
                    let va = values[*a];
                    let vb = values[*b];
                    super::nand_continuous::nand(va, vb)
                }
            };
            values.push(v);
        }
        values.last().copied()
    }

    pub fn nand_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| matches!(n, NandNode::Nand { .. }))
            .count()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

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
    fn dag_simple() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        let _out = dag.add_nand(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dag_not() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        dag.add_not(a);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);

        inputs.insert("a".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dag_and() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_and(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);

        inputs.insert("a".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn dag_or() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_or(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 0.0);
        inputs.insert("b".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);

        inputs.insert("a".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dag_xor() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_xor(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn dag_xnor() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_xnor(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn dag_nor() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_nor(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 0.0);
        inputs.insert("b".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);

        inputs.insert("a".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn dag_implies() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_implies(a, b);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), 1.0);
        inputs.insert("b".to_string(), 0.0);
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);

        inputs.insert("b".to_string(), 1.0);
        assert!((dag.evaluate(&inputs).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dag_constants() {
        let mut dag = NandDag::new();
        let a = dag.add_constant(1.0);
        let b = dag.add_constant(1.0);
        dag.add_nand(a, b);

        let inputs = HashMap::new();
        assert!((dag.evaluate(&inputs).unwrap() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn dag_missing_input() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        dag.add_input("b");
        dag.add_nand(a, a);

        let inputs = HashMap::new();
        assert!(dag.evaluate(&inputs).is_none());
    }

    #[test]
    fn dag_empty() {
        let dag = NandDag::new();
        assert!(dag.is_empty());
        let inputs = HashMap::new();
        assert!(dag.evaluate(&inputs).is_none());
    }

    #[test]
    fn dag_nand_counting() {
        let mut dag = NandDag::new();
        let a = dag.add_input("a");
        let b = dag.add_input("b");
        dag.add_xor(a, b);
        assert!(dag.nand_count() >= 4, "XOR should need >=4 NAND gates");
    }
}
