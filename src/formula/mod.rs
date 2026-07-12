pub struct FormulaRegistry;

impl FormulaRegistry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FormulaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_registry_creates() {
        let _r = FormulaRegistry::new();
    }
}
