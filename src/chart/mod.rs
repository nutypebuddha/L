pub struct Chart;

impl Chart {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_creates() {
        let _c = Chart::new();
    }
}
