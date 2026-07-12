pub struct EntityRegistry;

impl EntityRegistry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_registry_creates() {
        let _r = EntityRegistry::new();
    }
}
