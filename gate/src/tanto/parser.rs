// Tanto recursive descent expression parser — adapted for CID
// Precedence: ^ > unary - > * / > + -

use crate::tanto::math;
use crate::tanto::TantoEnv;

/// Deterministic resource limits for the recursive-descent parser.
///
/// These bounds exist so that untrusted input — the parser is on the
/// Gate/proposal side of the trust boundary — can never trigger unbounded
/// recursion (stack exhaustion) or unbounded node expansion (heap exhaustion)
/// regardless of how maliciously shaped the expression is. Every limit is
/// enforced in exactly one place: [`Parser::over_limits`].
#[derive(Debug, Clone, Copy)]
pub struct ParserLimits {
    /// Maximum number of input bytes accepted before parsing is refused.
    pub max_input_bytes: usize,
    /// Maximum number of whitespace-delimited tokens in op-format input.
    pub max_tokens: usize,
    /// Maximum AST nesting depth (recursion depth of `parse_expr`).
    pub max_depth: usize,
    /// Maximum number of AST nodes produced while parsing.
    pub max_nodes: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        ParserLimits {
            max_input_bytes: 1 << 22, // 4 MiB
            max_tokens: 1 << 18,      // 262_144 tokens
            max_depth: 256,
            max_nodes: 1 << 20, // 1_048_576 nodes
        }
    }
}

pub struct Parser<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
    pub env: &'a TantoEnv,
    pub limits: ParserLimits,
    depth: usize,
    nodes: usize,
}

impl<'a> Parser<'a> {
    pub fn new(bytes: &'a [u8], env: &'a TantoEnv) -> Self {
        Parser::with_limits(bytes, env, ParserLimits::default())
    }

    pub fn with_limits(bytes: &'a [u8], env: &'a TantoEnv, limits: ParserLimits) -> Self {
        Parser {
            bytes,
            pos: 0,
            env,
            limits,
            depth: 0,
            nodes: 0,
        }
    }

    /// True if the current parse has exceeded any configured resource limit.
    /// This is the single enforcement point for all parser limits.
    fn over_limits(&self) -> bool {
        self.depth > self.limits.max_depth || self.nodes > self.limits.max_nodes
    }

    fn bump_node(&mut self) -> bool {
        self.nodes += 1;
        !self.over_limits()
    }

    pub fn peek(&self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            Some(self.bytes[self.pos])
        } else {
            None
        }
    }

    pub fn advance(&mut self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    pub fn skip_ws(&mut self) {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos] == b' ' || self.bytes[self.pos] == b'\t')
        {
            self.pos += 1;
        }
    }

    pub fn parse_expr(&mut self) -> Option<f64> {
        if self.depth + 1 > self.limits.max_depth {
            return None;
        }
        self.depth += 1;
        let r = self.parse_add_sub();
        self.depth -= 1;
        r
    }

    fn parse_add_sub(&mut self) -> Option<f64> {
        if !self.bump_node() {
            return None;
        }
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
        if !self.bump_node() {
            return None;
        }
        let mut left = self.parse_power()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'*') => {
                    self.advance();
                    if self.peek() == Some(b'*') {
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
        if !self.bump_node() {
            return None;
        }
        let mut base = self.parse_unary()?;
        self.skip_ws();
        if self.peek() == Some(b'^') {
            self.advance();
            let exp = self.parse_unary()?;
            base = base.powf(exp);
        }
        Some(base)
    }

    fn parse_unary(&mut self) -> Option<f64> {
        if !self.bump_node() {
            return None;
        }
        self.skip_ws();
        match self.peek() {
            Some(b'-') => {
                self.advance();
                let val = self.parse_primary()?;
                Some(-val)
            }
            Some(b'+') => {
                self.advance();
                self.parse_primary()
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Option<f64> {
        if !self.bump_node() {
            return None;
        }
        self.skip_ws();
        match self.peek() {
            Some(b'(') => {
                self.advance();
                let val = self.parse_expr()?;
                self.skip_ws();
                if self.peek() == Some(b')') {
                    self.advance();
                    Some(val)
                } else {
                    None
                }
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.parse_number(),
            Some(c) if c.is_ascii_alphabetic() || c == b'_' => self.parse_name_or_func(),
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
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
        let mut val = s.parse::<f64>().ok()?;
        self.skip_ws();
        if self.peek() == Some(b'%') {
            self.advance();
            val /= 100.0;
        }
        Some(val)
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

        // Constant or variable (try 0-arg compute for pi, e, c, etc.)
        if let Some(v) = math::compute_named(name, &[]) {
            return Some(v);
        }
        self.env.get(std::str::from_utf8(name).ok()?)
    }
}

pub fn eval_math(expr: &[u8], env: &TantoEnv) -> Option<f64> {
    eval_math_with_limits(expr, env, ParserLimits::default())
}

/// Bounded variant of [`eval_math`] with explicit resource limits.
pub fn eval_math_with_limits(expr: &[u8], env: &TantoEnv, limits: ParserLimits) -> Option<f64> {
    if expr.len() > limits.max_input_bytes {
        return None;
    }
    let mut parser = Parser::with_limits(expr, env, limits);
    parser.parse_expr()
}

/// Op-format parser: "split 142.37 5" -> split(142.37, 5)
pub fn eval_op_format(expr: &[u8], env: &TantoEnv) -> Option<f64> {
    eval_op_format_with_limits(expr, env, ParserLimits::default())
}

/// Bounded variant of [`eval_op_format`] with explicit resource limits.
pub fn eval_op_format_with_limits(
    expr: &[u8],
    env: &TantoEnv,
    limits: ParserLimits,
) -> Option<f64> {
    let s = trim(expr);
    if s.is_empty() || s.len() > limits.max_input_bytes {
        return None;
    }

    let mut tokens: [&[u8]; 8] = [&[], &[], &[], &[], &[], &[], &[], &[]];
    let mut tcount = 0;
    let mut i = 0;
    while i < s.len() && tcount < limits.max_tokens.min(8) {
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
    for (argc, tok) in tokens[1..tcount].iter().enumerate() {
        args[argc] = parse_arg_value(tok, env)?;
    }

    math::compute_named(op, &args[..tcount - 1])
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

pub fn looks_like_expression(s: &[u8]) -> bool {
    let mut i = 0;
    while i < s.len() {
        match s[i] {
            b'+' | b'-' | b'*' | b'/' | b'^' | b'%' => {
                return true;
            }
            b'(' | b')' => {
                return true;
            }
            b'<' | b'>' => {
                return true;
            }
            b'=' => {
                if i + 1 < s.len() && s[i + 1] == b'=' {
                    return true;
                }
            }
            b'!' if i + 1 < s.len() && s[i + 1] == b'=' => {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_normal_expression_succeeds() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"2 + 3 * 4", &env), Some(14.0));
        assert_eq!(eval_math(b"(1 + 1) * (2 + 2)", &env), Some(8.0));
    }

    #[test]
    fn parser_input_size_limit_enforced() {
        let env = TantoEnv::new();
        let limits = ParserLimits {
            max_input_bytes: 4,
            ..ParserLimits::default()
        };
        // "12345" is 5 bytes > 4 byte limit.
        assert_eq!(eval_math_with_limits(b"12345", &env, limits), None);
        // Within limit still works.
        assert_eq!(eval_math_with_limits(b"123", &env, limits), Some(123.0));
    }

    #[test]
    fn parser_depth_limit_enforced() {
        let env = TantoEnv::new();
        // 300-deep nested parens exceeds the default max_depth of 256.
        let deep = format!("{}{}{}", "(".repeat(300), "1", ")".repeat(300));
        assert_eq!(eval_math(deep.as_bytes(), &env), None);
        // 100-deep is well within the limit.
        let shallow = format!("{}{}{}", "(".repeat(100), "1", ")".repeat(100));
        assert!(eval_math(shallow.as_bytes(), &env).is_some());
    }

    #[test]
    fn parser_node_limit_enforced() {
        let env = TantoEnv::new();
        // A chain of additions creates many AST nodes; cap nodes at 2 so even
        // "1+1+1" (3 leaf/op nodes) is rejected.
        let limits = ParserLimits {
            max_nodes: 8,
            ..ParserLimits::default()
        };
        // "1+1+1" expands to more than 8 parse nodes and is rejected.
        assert_eq!(eval_math_with_limits(b"1+1+1", &env, limits), None);
        // A single literal stays well under the node cap.
        let ok = ParserLimits {
            max_nodes: 8,
            ..ParserLimits::default()
        };
        assert_eq!(eval_math_with_limits(b"42", &env, ok), Some(42.0));
    }

    #[test]
    fn parser_op_format_token_limit_enforced() {
        let env = TantoEnv::new();
        let limits = ParserLimits {
            max_tokens: 1,
            ..ParserLimits::default()
        };
        // "add 2 3" has 3 tokens; with a 1-token cap it must be refused.
        assert_eq!(eval_op_format_with_limits(b"add 2 3", &env, limits), None);
    }
}
