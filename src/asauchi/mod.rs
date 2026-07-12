pub struct Asauchi;

impl Asauchi {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Asauchi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asauchi_creates() {
        let _a = Asauchi::new();
    }
}
