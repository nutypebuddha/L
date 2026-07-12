pub struct Ephemeris;

impl Ephemeris {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Ephemeris {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeris_creates() {
        let _e = Ephemeris::new();
    }
}
