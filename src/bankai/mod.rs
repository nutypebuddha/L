pub struct Bankai;

impl Bankai {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Bankai {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bankai_creates() {
        let _b = Bankai::new();
    }
}
