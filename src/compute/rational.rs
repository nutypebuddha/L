#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    pub num: i64,
    pub den: i64,
}

impl Fraction {
    pub fn new(num: i64, den: i64) -> Self {
        let mut f = Fraction { num, den };
        f.reduce();
        f
    }

    #[allow(clippy::cast_abs_to_unsigned)]
    pub fn reduce(&mut self) {
        if self.den == 0 {
            return;
        }
        if self.den < 0 {
            self.num = -self.num;
            self.den = -self.den;
        }
        let g = gcd(self.num.abs() as u64, self.den as u64) as i64;
        if g > 0 {
            self.num /= g;
            self.den /= g;
        }
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Fraction) -> Fraction {
        Fraction::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Fraction) -> Fraction {
        Fraction::new(
            self.num * other.den - other.num * self.den,
            self.den * other.den,
        )
    }

    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Fraction) -> Fraction {
        Fraction::new(self.num * other.num, self.den * other.den)
    }

    pub fn checked_div(self, other: Fraction) -> Option<Fraction> {
        if other.num == 0 {
            None
        } else {
            Some(Fraction::new(self.num * other.den, self.den * other.num))
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn parse_fraction(s: &str) -> Option<Fraction> {
    let s = s.trim();
    if let Some((num_s, den_s)) = s.split_once('/') {
        let num = num_s.trim().parse::<i64>().ok()?;
        let den = den_s.trim().parse::<i64>().ok()?;
        Some(Fraction::new(num, den))
    } else {
        let val = s.parse::<i64>().ok()?;
        Some(Fraction::new(val, 1))
    }
}

pub fn eval_rational(expr: &str, env: &super::TantoEnv) -> Option<Fraction> {
    let expr = expr.trim();
    if let Some(f) = parse_fraction(expr) {
        return Some(f);
    }

    for (name, &val) in env {
        if expr == name {
            let num = (val * 1e10) as i64;
            return Some(Fraction::new(num, 10_000_000_000));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_new() {
        let f = Fraction::new(6, 4);
        assert_eq!(f.num, 3);
        assert_eq!(f.den, 2);
    }

    #[test]
    fn test_fraction_add() {
        let a = Fraction::new(1, 3);
        let b = Fraction::new(1, 6);
        let c = a.add(b);
        assert_eq!(c.num, 1);
        assert_eq!(c.den, 2);
    }

    #[test]
    fn test_fraction_mul() {
        let a = Fraction::new(1, 3);
        let b = Fraction::new(3, 1);
        let c = a.mul(b);
        assert_eq!(c.num, 1);
        assert_eq!(c.den, 1);
    }

    #[test]
    fn test_parse_fraction() {
        assert_eq!(parse_fraction("1/3").unwrap(), Fraction::new(1, 3));
        assert_eq!(parse_fraction("5").unwrap(), Fraction::new(5, 1));
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 5), 1);
    }
}
