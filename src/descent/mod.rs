pub struct DescentEngine;

impl DescentEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&self, input: &str) -> Vec<String> {
        vec![input.to_lowercase()]
    }
}

impl Default for DescentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descent_engine_lowercase() {
        let engine = DescentEngine::new();
        let result = engine.process("HELLO");
        assert_eq!(result, vec!["hello"]);
    }
}
