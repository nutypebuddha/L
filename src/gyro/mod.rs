pub struct Gyro;

impl Gyro {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Gyro {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gyro_creates() {
        let _g = Gyro::new();
    }
}
