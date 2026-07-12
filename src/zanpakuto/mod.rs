pub struct Zanpakuto;

impl Zanpakuto {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Zanpakuto {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zanpakuto_creates() {
        let _z = Zanpakuto::new();
    }
}
