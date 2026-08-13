//! Tanto recursive descent expression parser — f64 evaluation.
//!
//! Precedence (lowest to highest):
//!   1. `+`, `-` (add/sub)
//!   2. `*`, `/`, `%` (mul/div/mod)
//!   3. `^` (power, right-associative via unary)
//!   4. unary `-`, `+`
//!   5. Primary: numbers, variables, functions, parentheses
//!
//! All parsing is done on byte slices — zero allocation beyond the input.

use super::math;
use super::TantoEnv;

/// Recursive descent parser for arithmetic expressions.
pub struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    env: &'a TantoEnv,
}

impl<'a> Parser<'a> {
    pub fn new(bytes: &'a [u8], env: &'a TantoEnv) -> Self {
        Parser { bytes, pos: 0, env }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            Some(self.bytes[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos] == b' ' || self.bytes[self.pos] == b'\t')
        {
            self.pos += 1;
        }
    }

    /// Parse a full expression (entry point).
    pub fn parse_expr(&mut self) -> Option<f64> {
        self.parse_add_sub()
    }

    // ─── Precedence levels ───────────────────────────────────────

    fn parse_add_sub(&mut self) -> Option<f64> {
        let mut left = self.parse_mul_div()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'+') => {
                    self.advance();
                    let right = self.parse_mul_div()?;
                    left += right;
                }
                Some(b'-') => {
                    self.advance();
                    let right = self.parse_mul_div()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_mul_div(&mut self) -> Option<f64> {
        let mut left = self.parse_power()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'*') => {
                    self.advance();
                    if self.peek() == Some(b'*') {
                        // ** for power (Python-style)
                        self.advance();
                        let right = self.parse_power()?;
                        left = left.powf(right);
                    } else {
                        let right = self.parse_power()?;
                        left *= right;
                    }
                }
                Some(b'/') => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return None;
                    }
                    left /= right;
                }
                Some(b'%') => {
                    // Binary % operator: modulo (like CID reference)
                    // Postfix % on numbers is handled in parse_primary (no whitespace)
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return None;
                    }
                    left %= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_power(&mut self) -> Option<f64> {
        let base = self.parse_unary()?;
        self.skip_ws();
        if self.peek() == Some(b'^') {
            self.advance();
            let exp = self.parse_power()?; // Right-associative: recurse
            Some(base.powf(exp))
        } else {
            Some(base)
        }
    }

    fn parse_unary(&mut self) -> Option<f64> {
        self.skip_ws();
        match self.peek() {
            Some(b'-') => {
                self.advance();
                let val = self.parse_unary()?; // Recurse for multiple unary operators
                Some(-val)
            }
            Some(b'+') => {
                self.advance();
                self.parse_unary() // Recurse for multiple unary operators
            }
            _ => self.parse_primary(),
        }
    }

    // ─── Primary expressions ─────────────────────────────────────

    fn parse_primary(&mut self) -> Option<f64> {
        self.skip_ws();
        let mut val = match self.peek() {
            Some(b'(') => {
                self.advance();
                let val = self.parse_expr()?;
                self.skip_ws();
                if self.peek() == Some(b')') {
                    self.advance();
                    val
                } else {
                    return None;
                }
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.parse_number()?,
            Some(c) if c.is_ascii_alphabetic() || c == b'_' => self.parse_name_or_func()?,
            _ => return None,
        };

        // Postfix percentage: immediately after number with NO space
        // e.g., "50%" -> 0.5, but "10 % 3" triggers binary modulo
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'%' {
            self.pos += 1;
            val /= 100.0;
        }

        Some(val)
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        // Integer part
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        // Decimal part
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        // Scientific notation
        if self.pos < self.bytes.len()
            && (self.bytes[self.pos] == b'e' || self.bytes[self.pos] == b'E')
        {
            self.pos += 1;
            if self.pos < self.bytes.len()
                && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-')
            {
                self.pos += 1;
            }
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let raw = &self.bytes[start..self.pos];
        let s = std::str::from_utf8(raw).ok()?;
        s.parse::<f64>().ok()
    }

    fn parse_name_or_func(&mut self) -> Option<f64> {
        let start = self.pos;
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_alphanumeric() || self.bytes[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let name = &self.bytes[start..self.pos];
        self.skip_ws();

        // Function call?
        if self.peek() == Some(b'(') {
            self.advance();
            let mut args = [0.0_f64; 8];
            let mut argc = 0;
            self.skip_ws();
            if self.peek() != Some(b')') {
                loop {
                    if argc >= 8 {
                        return None;
                    }
                    args[argc] = self.parse_expr()?;
                    argc += 1;
                    self.skip_ws();
                    if self.peek() == Some(b',') {
                        self.advance();
                        continue;
                    }
                    break;
                }
            }
            self.skip_ws();
            if self.peek() != Some(b')') {
                return None;
            }
            self.advance();
            return math::compute_named(name, &args[..argc]);
        }

        // Constant or variable (try 0-arg constants like pi, e)
        if let Some(v) = math::compute_named(name, &[]) {
            return Some(v);
        }
        self.env.get(std::str::from_utf8(name).ok()?)
    }
}

/// Evaluate a byte-slice expression as f64.
/// Returns None if the expression is empty or has unconsumed trailing input.
pub fn eval_math(expr: &[u8], env: &TantoEnv) -> Option<f64> {
    let mut parser = Parser::new(expr, env);
    let result = parser.parse_expr()?;
    // Ensure all input was consumed — partial parses are invalid
    parser.skip_ws();
    if parser.pos < parser.bytes.len() {
        return None;
    }
    Some(result)
}

/// Op-format parser: `"split 142.37 5"` → `split(142.37, 5)`
pub fn eval_op_format(expr: &[u8], env: &TantoEnv) -> Option<f64> {
    let s = trim(expr);
    if s.is_empty() {
        return None;
    }

    let mut tokens: [&[u8]; 8] = [&[], &[], &[], &[], &[], &[], &[], &[]];
    let mut tcount = 0;
    let mut i = 0;
    while i < s.len() && tcount < 8 {
        while i < s.len() && (s[i] == b' ' || s[i] == b'\t') {
            i += 1;
        }
        if i >= s.len() {
            break;
        }
        let start = i;
        while i < s.len() && s[i] != b' ' && s[i] != b'\t' {
            i += 1;
        }
        tokens[tcount] = &s[start..i];
        tcount += 1;
    }
    if tcount == 0 {
        return None;
    }

    let op = tokens[0];
    let mut args = [0.0_f64; 4];
    let mut argc = 0;
    for tok in &tokens[1..tcount] {
        args[argc] = parse_arg_value(tok, env)?;
        argc += 1;
    }

    math::compute_named(op, &args[..argc])
}

fn parse_arg_value(tok: &[u8], env: &TantoEnv) -> Option<f64> {
    // Pipeline placeholder (handled at pipeline level)
    if tok == b"_" {
        return None;
    }
    // Try name lookup
    let name = std::str::from_utf8(tok).ok()?;
    if let Some(v) = env.get(name) {
        return Some(v);
    }
    // Numeric literal
    name.parse::<f64>().ok()
}

/// Trim whitespace from a byte slice.
pub fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len()
        && (s[start] == b' ' || s[start] == b'\t' || s[start] == b'\n' || s[start] == b'\r')
    {
        start += 1;
    }
    let mut end = s.len();
    while end > start
        && (s[end - 1] == b' ' || s[end - 1] == b'\t' || s[end - 1] == b'\n' || s[end - 1] == b'\r')
    {
        end -= 1;
    }
    &s[start..end]
}

/// Check if text looks like it contains math operators.
pub fn looks_like_expression(s: &[u8]) -> bool {
    let mut i = 0;
    while i < s.len() {
        match s[i] {
            b'+' | b'-' | b'*' | b'/' | b'^' | b'%' => return true,
            b'(' | b')' => return true,
            b'<' | b'>' => return true,
            b'=' if i + 1 < s.len() && s[i + 1] == b'=' => return true,
            b'!' if i + 1 < s.len() && s[i + 1] == b'=' => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> TantoEnv {
        TantoEnv::new()
    }

    #[test]
    fn test_simple_add() {
        assert!((eval_math(b"2 + 3", &env()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_simple_sub() {
        assert!((eval_math(b"5 - 3", &env()).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_simple_mul() {
        assert!((eval_math(b"3 * 4", &env()).unwrap() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_simple_div() {
        assert!((eval_math(b"10 / 2", &env()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_power() {
        assert!((eval_math(b"2 ^ 3", &env()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_modulo() {
        let result = eval_math(b"10 % 3", &env()).unwrap();
        println!("10 % 3 = {}", result);
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_unary_minus() {
        assert!((eval_math(b"-5", &env()).unwrap() + 5.0).abs() < 1e-10);
        assert!((eval_math(b"--5", &env()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_precedence() {
        // 2 + 3 * 4 = 14 (not 20)
        assert!((eval_math(b"2 + 3 * 4", &env()).unwrap() - 14.0).abs() < 1e-10);
        // (2 + 3) * 4 = 20
        assert!((eval_math(b"(2 + 3) * 4", &env()).unwrap() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_power_precedence() {
        // 2 ^ 3 ^ 2 = 2 ^ (3 ^ 2) = 2 ^ 9 = 512 (right-associative, standard math)
        let result = eval_math(b"2 ^ 3 ^ 2", &env()).unwrap();
        assert!((result - 512.0).abs() < 1e-10);
    }

    #[test]
    fn test_star_star_power() {
        assert!((eval_math(b"2 ** 3", &env()).unwrap() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_parentheses() {
        assert!((eval_math(b"((1 + 2) * 3)", &env()).unwrap() - 9.0).abs() < 1e-10);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_decimal() {
        assert!((eval_math(b"3.14 * 2", &env()).unwrap() - 6.28).abs() < 1e-10);
    }

    #[test]
    fn test_scientific() {
        let result = eval_math(b"1e3", &env()).unwrap();
        assert!((result - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn test_percent() {
        // 50% = 0.5
        assert!((eval_math(b"50%", &env()).unwrap() - 0.5).abs() < 1e-10);
        // 100 + 10% = 100.1 (postfix % converts 10 to 0.1)
        assert!((eval_math(b"100 + 10%", &env()).unwrap() - 100.1).abs() < 1e-10);
    }

    #[test]
    fn test_sqrt_function() {
        assert!((eval_math(b"sqrt(9)", &env()).unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_abs_function() {
        assert!((eval_math(b"abs(-5)", &env()).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_sin_cos() {
        let env = env();
        let sin_val = eval_math(b"sin(0)", &env).unwrap();
        assert!((sin_val - 0.0).abs() < 1e-15);
        let cos_val = eval_math(b"cos(0)", &env).unwrap();
        assert!((cos_val - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_multi_arg_function() {
        let env = env();
        assert!((eval_math(b"min(3, 7, 1, 9)", &env).unwrap() - 1.0).abs() < 1e-10);
        assert!((eval_math(b"max(3, 7, 1, 9)", &env).unwrap() - 9.0).abs() < 1e-10);
        assert!((eval_math(b"hypot(3, 4)", &env).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_variable_lookup() {
        let mut env = env();
        env.set("mass", 5.0);
        env.set("velocity", 10.0);
        let ke = eval_math(b"0.5 * mass * velocity ^ 2", &env).unwrap();
        assert!((ke - 250.0).abs() < 1e-10);
    }

    #[test]
    fn test_constant_pi() {
        let env = env();
        let result = eval_math(b"pi", &env).unwrap();
        assert!((result - std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn test_div_by_zero_returns_none() {
        assert!(eval_math(b"1 / 0", &env()).is_none());
    }

    #[test]
    fn test_empty_expr() {
        assert!(eval_math(b"", &env()).is_none());
    }

    #[test]
    fn test_mismatched_parens() {
        assert!(eval_math(b"(1 + 2", &env()).is_none());
    }

    #[test]
    fn test_op_format_split() {
        let env = env();
        let result = eval_op_format(b"split 100 4", &env).unwrap();
        assert!((result - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_looks_like_expression() {
        assert!(looks_like_expression(b"2 + 2"));
        assert!(looks_like_expression(b"x == y"));
        assert!(!looks_like_expression(b"hello world"));
    }

    #[test]
    fn test_chain_ops() {
        let env = env();
        // 1 + 2 * 3 - 4 / 2 = 1 + 6 - 2 = 5
        assert!((eval_math(b"1 + 2 * 3 - 4 / 2", &env).unwrap() - 5.0).abs() < 1e-10);
    }
}
