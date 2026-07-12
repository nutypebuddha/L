pub struct Shikai;

impl Shikai {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Shikai {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shikai_creates() {
        let _s = Shikai::new();
    }
}
