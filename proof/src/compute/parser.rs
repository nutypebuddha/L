use super::TantoEnv;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    LParen,
    RParen,
    Comma,
    Func(String),
    Ident(String),
    // T59: relational operators. Deliberately not `Eq` — a bare `=` is not
    // part of the Tanto grammar and stays handled at the string level in
    // math_gate.rs (equation-balance checking), which already works
    // correctly. Adding these five closes the gap where `<`/`>` silently
    // dropped everything after them and `<=`/`>=`/`!=` were misread as `=`
    // by math_gate.rs's `token.find('=')` (see check_equation_correctness).
    Lt,
    Gt,
    Le,
    Ge,
    Ne,
}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Self {
        Lexer { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos] == b' ' {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }
        let c = self.input[self.pos];
        match c {
            b'+' => {
                self.pos += 1;
                Some(Token::Plus)
            }
            b'-' => {
                self.pos += 1;
                Some(Token::Minus)
            }
            b'*' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == b'*' {
                    self.pos += 1;
                    Some(Token::Pow)
                } else {
                    Some(Token::Mul)
                }
            }
            b'/' => {
                self.pos += 1;
                Some(Token::Div)
            }
            b'%' => {
                self.pos += 1;
                Some(Token::Mod)
            }
            b'^' => {
                self.pos += 1;
                Some(Token::Pow)
            }
            b'(' => {
                self.pos += 1;
                Some(Token::LParen)
            }
            b')' => {
                self.pos += 1;
                Some(Token::RParen)
            }
            b',' => {
                self.pos += 1;
                Some(Token::Comma)
            }
            b'<' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == b'=' {
                    self.pos += 1;
                    Some(Token::Le)
                } else {
                    Some(Token::Lt)
                }
            }
            b'>' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == b'=' {
                    self.pos += 1;
                    Some(Token::Ge)
                } else {
                    Some(Token::Gt)
                }
            }
            b'!' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == b'=' {
                    self.pos += 2;
                    Some(Token::Ne)
                } else {
                    // Lone `!` (no boolean-not in this grammar): do NOT
                    // advance past it. Leaving `pos` pointing at the
                    // unconsumed byte — rather than past it — means the
                    // end-of-input check in eval_math/parse_math correctly
                    // reads this as "stopped early," not "reached EOF
                    // cleanly," even when the `!` is the last byte in the
                    // string (e.g. "5!" with nothing after it).
                    None
                }
            }
            b'0'..=b'9' | b'.' => self.read_number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.read_ident(),
            _ => None,
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if self.pos < self.input.len()
            && (self.input[self.pos] == b'e' || self.input[self.pos] == b'E')
        {
            self.pos += 1;
            if self.pos < self.input.len()
                && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-')
            {
                self.pos += 1;
            }
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let s = std::str::from_utf8(&self.input[start..self.pos]).ok()?;
        let val = s.parse::<f64>().ok()?;
        Some(Token::Num(val))
    }

    fn read_ident(&mut self) -> Option<Token> {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.input[start..self.pos]).ok()?;
        let funcs: &[&str] = &[
            "sqrt",
            "sin",
            "cos",
            "tan",
            "asin",
            "acos",
            "atan",
            "atan2",
            "abs",
            "exp",
            "ln",
            "log10",
            "log2",
            "hypot",
            "pow",
            "round",
            "floor",
            "ceil",
            "min",
            "max",
            "clamp",
            "sum",
            "avg",
            "erf",
            "log",
            "diff",
            "factorial",
            "gcd",
            "gauss_inv",
            "rad2deg",
            "deg2rad",
            "norm",
            "f_to_c",
            "c_to_f",
            "c_to_k",
            "f_to_k",
            "k_to_c",
            "k_to_f",
            "mi_to_km",
            "km_to_mi",
            "mph_to_kmh",
            "kmh_to_mph",
            "lb_to_kg",
            "kg_to_lb",
            "ft_to_m",
            "in_to_cm",
            "mph_to_ms",
            "ms_to_mph",
        ];
        if funcs.contains(&s) {
            Some(Token::Func(s.to_string()))
        } else {
            Some(Token::Ident(s.to_string()))
        }
    }
}

/// Epsilon for float "close enough to equal" comparisons — matches the
/// 1e-10 tight threshold `check_equation_correctness` (math_gate.rs) already
/// uses for `=`. Used here so `<=`/`>=`/`!=` agree with `=` at the boundary
/// instead of being tripped by ordinary float representation noise, e.g.
/// `0.1 + 0.2 <= 0.3` should read as true, not false because 0.1+0.2 comes
/// out as 0.30000000000000004 in raw IEEE-754.
const CMP_EPS: f64 = 1e-10;

#[derive(Clone, Copy)]
enum CmpOp {
    Lt,
    Gt,
    Le,
    Ge,
    Ne,
}

impl CmpOp {
    fn eval(self, l: f64, r: f64) -> bool {
        let close = (l - r).abs() < CMP_EPS;
        match self {
            CmpOp::Lt => l < r && !close,
            CmpOp::Gt => l > r && !close,
            CmpOp::Le => l < r || close,
            CmpOp::Ge => l > r || close,
            CmpOp::Ne => !close,
        }
    }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    env: &'a TantoEnv,
    /// When true, the parser only checks grammar structure (balanced, known
    /// operators/functions) and tolerates division/modulo by zero — used by
    /// `parse_math` for corpus expression validation where a concrete finite
    /// value is irrelevant.
    structural: bool,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, env: &'a TantoEnv) -> Self {
        Parser {
            tokens,
            pos: 0,
            env,
            structural: false,
        }
    }

    fn new_structural(tokens: Vec<Token>, env: &'a TantoEnv) -> Self {
        Parser {
            tokens,
            pos: 0,
            env,
            structural: true,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    /// T59: comparisons bind loosest, one level above +/-, matching the
    /// convention in every C-family/Python-family grammar (arithmetic is
    /// evaluated on both sides, then compared). Deliberately non-chaining —
    /// "1 < 2 < 3" parses `1 < 2` (=1.0) and then stops, leaving `< 3`
    /// unconsumed, which the full-consumption check in eval_math/parse_math
    /// correctly rejects as malformed rather than silently accepting a
    /// Python-style chained comparison this grammar was never designed for.
    fn parse_expr(&mut self) -> Option<f64> {
        let left = self.parse_additive()?;
        let op = match self.peek() {
            Some(Token::Lt) => CmpOp::Lt,
            Some(Token::Gt) => CmpOp::Gt,
            Some(Token::Le) => CmpOp::Le,
            Some(Token::Ge) => CmpOp::Ge,
            Some(Token::Ne) => CmpOp::Ne,
            _ => return Some(left),
        };
        self.advance();
        let right = self.parse_additive()?;
        Some(if op.eval(left, right) { 1.0 } else { 0.0 })
    }

    fn parse_additive(&mut self) -> Option<f64> {
        let mut left = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left += right;
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut left = self.parse_power()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Mul => {
                    self.advance();
                    let right = self.parse_power()?;
                    left *= right;
                }
                Token::Div => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        if self.structural {
                            // Division by zero is irrelevant in structural mode;
                            // leave `left` unchanged so the parse continues.
                        } else {
                            return None;
                        }
                    } else {
                        left /= right;
                    }
                }
                Token::Mod => {
                    self.advance();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        if self.structural {
                            // Modulo by zero is irrelevant in structural mode.
                        } else {
                            return None;
                        }
                    } else {
                        left %= right;
                    }
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_power(&mut self) -> Option<f64> {
        let base = self.parse_unary()?;
        if let Some(Token::Pow) = self.peek() {
            self.advance();
            let exp = self.parse_power()?; // Right-associative: 2^3^2 = 2^(3^2) = 512
            Some(base.powf(exp))
        } else {
            Some(base)
        }
    }

    fn parse_unary(&mut self) -> Option<f64> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let val = self.parse_atom()?;
                Some(-val)
            }
            Some(Token::Plus) => {
                self.advance();
                self.parse_atom()
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Option<f64> {
        let tok = self.advance()?;
        match tok {
            Token::Num(val) => Some(val),
            Token::LParen => {
                let val = self.parse_expr()?;
                if self.advance() != Some(Token::RParen) {
                    return None;
                }
                Some(val)
            }
            Token::Func(name) => {
                if self.advance() != Some(Token::LParen) {
                    return None;
                }
                let mut args = Vec::new();
                if self.peek() != Some(Token::RParen) {
                    args.push(self.parse_expr()?);
                    while self.peek() == Some(Token::Comma) {
                        self.advance();
                        args.push(self.parse_expr()?);
                    }
                }
                if self.advance() != Some(Token::RParen) {
                    return None;
                }
                eval_func(&name, &args)
            }
            Token::Ident(name) => self.env.get(&name).copied(),
            _ => None,
        }
    }
}

fn eval_func(name: &str, args: &[f64]) -> Option<f64> {
    match name {
        "sqrt" => args.first().map(|x| x.sqrt()),
        "sin" => args.first().map(|x| x.sin()),
        "cos" => args.first().map(|x| x.cos()),
        "tan" => args.first().map(|x| x.tan()),
        "asin" => args.first().map(|x| x.asin()),
        "acos" => args.first().map(|x| x.acos()),
        "atan" => args.first().map(|x| x.atan()),
        "atan2" => {
            if args.len() >= 2 {
                Some(args[0].atan2(args[1]))
            } else {
                None
            }
        }
        "abs" => args.first().map(|x| x.abs()),
        "exp" => args.first().map(|x| x.exp()),
        "ln" => args.first().map(|x| x.ln()),
        "log10" => args.first().map(|x| x.log10()),
        "log2" => args.first().map(|x| x.log2()),
        "hypot" => {
            if args.len() >= 2 {
                Some(args[0].hypot(args[1]))
            } else {
                None
            }
        }
        "pow" => {
            if args.len() >= 2 {
                Some(args[0].powf(args[1]))
            } else {
                None
            }
        }
        "round" => args.first().map(|x| x.round()),
        "floor" => args.first().map(|x| x.floor()),
        "ceil" => args.first().map(|x| x.ceil()),
        "min" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().cloned().fold(f64::INFINITY, f64::min))
            }
        }
        "max" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
            }
        }
        "clamp" => {
            if args.len() >= 3 {
                let val = args[0].max(args[1]).min(args[2]);
                Some(val)
            } else {
                None
            }
        }
        "sum" => Some(args.iter().sum()),
        "avg" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().sum::<f64>() / args.len() as f64)
            }
        }
        "mean" => {
            if args.is_empty() {
                None
            } else {
                Some(args.iter().sum::<f64>() / args.len() as f64)
            }
        }
        "erf" => args.first().map(|x| erf(*x)),
        "log" => {
            if args.is_empty() {
                None
            } else if args.len() == 1 {
                Some(args[0].ln())
            } else {
                Some(args[0].ln() / args[1].ln())
            }
        }
        "diff" => {
            if args.len() < 2 {
                Some(0.0)
            } else {
                Some(args[1] - args[0])
            }
        }
        "factorial" => args.first().map(|x| factorial(*x)),
        "gcd" => {
            if args.len() < 2 {
                None
            } else {
                Some(gcd(args[0], args[1]))
            }
        }
        "gauss_inv" => args.first().map(|p| gauss_inv(*p)),
        "rad2deg" => args.first().map(|x| x.to_degrees()),
        "deg2rad" => args.first().map(|x| x.to_radians()),
        "norm" => {
            if args.len() >= 2 {
                Some((args[0] * args[0] + args[1] * args[1]).sqrt())
            } else {
                None
            }
        }
        "f_to_c" => args.first().map(|f| (f - 32.0) * 5.0 / 9.0),
        "c_to_f" => args.first().map(|c| c * 9.0 / 5.0 + 32.0),
        "c_to_k" => args.first().map(|c| c + 273.15),
        "f_to_k" => args.first().map(|f| (f - 32.0) * 5.0 / 9.0 + 273.15),
        "k_to_c" => args.first().map(|k| k - 273.15),
        "k_to_f" => args.first().map(|k| (k - 273.15) * 9.0 / 5.0 + 32.0),
        "mi_to_km" => args.first().map(|mi| mi * 1.609344),
        "km_to_mi" => args.first().map(|km| km / 1.609344),
        "mph_to_kmh" => args.first().map(|mph| mph * 1.609344),
        "kmh_to_mph" => args.first().map(|kmh| kmh / 1.609344),
        "lb_to_kg" => args.first().map(|lb| lb * 0.453592),
        "kg_to_lb" => args.first().map(|kg| kg / 0.453592),
        "ft_to_m" => args.first().map(|ft| ft * 0.3048),
        "in_to_cm" => args.first().map(|inches| inches * 2.54),
        "mph_to_ms" => args.first().map(|mph| mph * 0.44704),
        "ms_to_mph" => args.first().map(|ms| ms / 0.44704),
        _ => None,
    }
}

/// T59: previously returned whatever `parse_expr` produced from a prefix of
/// the tokens, even when tokens remained unconsumed — so `"9.11 < 9.9"`
/// (where `<` wasn't yet a recognized token) silently evaluated as `9.11`
/// with the rest of the string discarded. Now mirrors the full-consumption
/// check `parse_math` already used below for corpus validation: every token
/// must be consumed or the whole expression is rejected. Every call site
/// (verify.rs, pipeline.rs, mod.rs, math_gate.rs) already falls back to an
/// alternate parser (`eval_op_format` or a tokenizer) on `None`, so this
/// does not regress any of them — it just stops a truncated prefix from
/// masquerading as a complete answer.
pub fn eval_math(input: &[u8], env: &TantoEnv) -> Option<f64> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token() {
        tokens.push(tok);
    }
    // Two different ways this loop can stop early, both indistinguishable
    // from a clean EOF unless checked explicitly: next_token() returns None
    // both when input is exhausted AND when it hits a byte it doesn't
    // recognize (e.g. '@') or an incomplete operator (a lone trailing '!').
    // Comparing the lexer's final position against the input length tells
    // them apart — a genuine EOF always leaves pos == input.len().
    if lexer.pos != input.len() {
        return None;
    }
    if tokens.is_empty() {
        return None;
    }
    let mut parser = Parser::new(tokens, env);
    let val = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return None;
    }
    Some(val)
}

/// Structural-only parse: returns `Some(())` iff the entire input is
/// well-formed Tanto grammar (balanced, known operators/functions, every
/// token consumed), independent of whether it would evaluate to a finite
/// number. Used by `laverna corpus validate` to detect malformed corpus
/// expressions without false-positiving on legitimate free variables or
/// division-by-zero special cases.
pub fn parse_math(input: &[u8], env: &TantoEnv) -> Option<()> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token() {
        tokens.push(tok);
    }
    if lexer.pos != input.len() {
        return None;
    }
    if tokens.is_empty() {
        return None;
    }
    let mut parser = Parser::new_structural(tokens, env);
    parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return None;
    }
    Some(())
}

/// Pure function: Abramowitz & Stegun 7.1.26 approximation of the error
/// function, max error ~1.5e-7.
fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-x * x).exp();
    sign * y
}

/// Pure function: factorial of a non-negative integer, computed iteratively.
/// Non-integer and negative inputs fall back to the gamma extension
/// `Γ(n+1)` is not available, so we clamp to the nearest non-negative
/// integer to keep the result finite and deterministic.
fn factorial(x: f64) -> f64 {
    if x < 0.0 {
        return f64::NAN;
    }
    let n = x.round() as u64;
    let mut accumulator = 1u64;
    for i in 2..=n {
        accumulator = accumulator.saturating_mul(i);
        if accumulator == u64::MAX {
            break;
        }
    }
    accumulator as f64
}

/// Pure function: greatest common divisor via the Euclidean algorithm.
fn gcd(a: f64, b: f64) -> f64 {
    let (mut a, mut b) = (a.abs().round() as i64, b.abs().round() as i64);
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.unsigned_abs() as f64
}

/// Pure function: inverse of the standard normal CDF (probit) via the
/// Acklam rational approximation, relative error < 1.15e-9.
#[allow(clippy::excessive_precision)]
fn gauss_inv(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let a = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    let b = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    let c = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    let d = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= phigh {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

pub fn eval_op_format(input: &[u8], env: &TantoEnv) -> Option<f64> {
    let input = trim(input);
    if input.is_empty() {
        return None;
    }

    let (op, rest) = split_token(input)?;
    let op_str = std::str::from_utf8(op).ok()?;
    let rest = trim(rest);

    let mut args = Vec::new();
    if !rest.is_empty() {
        for token in rest.split(|&b| b == b' ') {
            let token = trim(token);
            if token.is_empty() {
                continue;
            }
            if let Some(v) = eval_math(token, env) {
                args.push(v);
            }
        }
    }

    super::math::compute_named(op_str, &args)
}

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && (s[start] == b' ' || s[start] == b'\t') {
        start += 1;
    }
    let mut end = s.len();
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\t') {
        end -= 1;
    }
    &s[start..end]
}

fn split_token(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let s = trim(s);
    if s.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < s.len() && s[i] != b' ' && s[i] != b'\t' {
        i += 1;
    }
    if i >= s.len() {
        return Some((s, &[]));
    }
    Some((&s[..i], &s[i..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_math_basic() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"2+3", &env), Some(5.0));
        assert_eq!(eval_math(b"10-4", &env), Some(6.0));
        assert_eq!(eval_math(b"3*4", &env), Some(12.0));
        assert_eq!(eval_math(b"12/4", &env), Some(3.0));
    }

    #[test]
    fn test_eval_math_precedence() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"2+3*4", &env), Some(14.0));
        assert_eq!(eval_math(b"(2+3)*4", &env), Some(20.0));
    }

    #[test]
    fn test_eval_math_negative() {
        let env = TantoEnv::new();
        assert_eq!(eval_math(b"-5", &env), Some(-5.0));
        assert_eq!(eval_math(b"-2^2", &env), Some(4.0));
    }

    #[test]
    fn test_eval_math_functions() {
        let env = TantoEnv::new();
        assert!((eval_math(b"sqrt(144)", &env).unwrap() - 12.0).abs() < 1e-10);
        assert!((eval_math(b"sin(0)", &env).unwrap() - 0.0).abs() < 1e-10);
        assert!((eval_math(b"cos(0)", &env).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_op_format() {
        let env = TantoEnv::new();
        assert_eq!(eval_op_format(b"add 2 3", &env), Some(5.0));
        assert_eq!(eval_op_format(b"mul 3 4", &env), Some(12.0));
    }

    // ── D3 regression: exponent chain must not drop terms ──

    #[test]
    fn d3_exponent_chain_right_associative() {
        let env = TantoEnv::new();
        // 2^3^2 = 2^(3^2) = 2^9 = 512 (right-associative)
        let v = eval_math(b"2^3^2", &env).unwrap();
        assert!((v - 512.0).abs() < 1e-6, "expected 512, got {v}");

        // ** is the same operator
        let v = eval_math(b"2**3**2", &env).unwrap();
        assert!((v - 512.0).abs() < 1e-6, "expected 512, got {v}");

        // Explicit parens still work
        assert!((eval_math(b"(2^3)^2", &env).unwrap() - 64.0).abs() < 1e-6);
        assert!((eval_math(b"2^(3^2)", &env).unwrap() - 512.0).abs() < 1e-6);
        assert!((eval_math(b"2^3", &env).unwrap() - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_math_functions() {
        let env = TantoEnv::new();
        // erf — Abramowitz & Stegun approximation
        assert!((eval_math(b"erf(0)", &env).unwrap() - 0.0).abs() < 1e-6);
        assert!((eval_math(b"erf(1)", &env).unwrap() - 0.84270079).abs() < 1e-4);
        // log — natural log single-arg, base-2 two-arg
        assert!((eval_math(b"log(2.718281828)", &env).unwrap() - 1.0).abs() < 1e-6);
        assert!((eval_math(b"log(8, 2)", &env).unwrap() - 3.0).abs() < 1e-6);
        // diff — forward difference (args[1] - args[0]); 0.0 for a single arg
        assert!((eval_math(b"diff(5, 2)", &env).unwrap() + 3.0).abs() < 1e-9);
        assert_eq!(eval_math(b"diff(7)", &env), Some(0.0));
        // factorial — iterative
        assert_eq!(eval_math(b"factorial(5)", &env), Some(120.0));
        assert_eq!(eval_math(b"factorial(0)", &env), Some(1.0));
        // gcd — Euclidean
        assert_eq!(eval_math(b"gcd(48, 18)", &env), Some(6.0));
        // gauss_inv — probit (Acklam); gauss_inv(0.5) ≈ 0
        assert!(eval_math(b"gauss_inv(0.5)", &env).unwrap().abs() < 1e-6);
        assert!((eval_math(b"gauss_inv(0.9772)", &env).unwrap() - 2.0).abs() < 1e-2);
    }

    #[test]
    fn test_structural_parse_math() {
        let mut env = TantoEnv::new();
        for id in ["a", "b", "w", "x", "y", "z", "unknown_fn"] {
            env.insert(id.to_string(), 1.0);
        }
        // Division by zero is tolerated in structural mode (whole expr valid).
        assert!(parse_math(b"a / (b - b)", &env).is_some());
        // Balanced, known operators, every token consumed.
        assert!(parse_math(b"x * (y + z) / w", &env).is_some());
        // Trailing garbage is rejected.
        assert!(parse_math(b"a + b )", &env).is_none());
        // Unknown function name (treated as ident) leaves trailing call → invalid.
        assert!(parse_math(b"unknown_fn(x)", &env).is_none());
        // Empty input rejected.
        assert!(parse_math(b"", &env).is_none());
    }

    // ── T59: relational operators ──
    //
    // Before this fix, `<` and `>` weren't tokens at all: the lexer stopped
    // at the unrecognized byte and the parser happily returned whatever it
    // had parsed so far, silently discarding everything after the operator.
    // `eval_math(b"9.11 < 9.9", ..)` returned `Some(9.11)`. Every case below
    // is a direct repro from that session, kept as the regression guard.

    #[test]
    fn t59_relational_operators_evaluate_correctly() {
        let env = TantoEnv::new();
        // The original repro: both directions of the same decimal pair.
        assert_eq!(eval_math(b"9.11 < 9.9", &env), Some(1.0));
        assert_eq!(eval_math(b"9.11 > 9.9", &env), Some(0.0));
        // >=, <=, != were being silently rewritten as `=` by math_gate.rs's
        // `token.find('=')` (it matches the `=` *inside* those operators).
        // At the parser level they weren't tokens at all until now.
        assert_eq!(eval_math(b"5 >= 3", &env), Some(1.0));
        assert_eq!(eval_math(b"3 >= 5", &env), Some(0.0));
        assert_eq!(eval_math(b"3 <= 5", &env), Some(1.0));
        assert_eq!(eval_math(b"5 <= 3", &env), Some(0.0));
        assert_eq!(eval_math(b"5 != 3", &env), Some(1.0));
        assert_eq!(eval_math(b"5 != 5", &env), Some(0.0));
        // Boundary: <=/>= must accept exact equality.
        assert_eq!(eval_math(b"5 <= 5", &env), Some(1.0));
        assert_eq!(eval_math(b"5 >= 5", &env), Some(1.0));
    }

    #[test]
    fn t59_comparison_uses_same_epsilon_as_equation_balance() {
        let env = TantoEnv::new();
        // 0.1 + 0.2 is 0.30000000000000004 in raw IEEE-754. A strict `<=`
        // would read this as false. check_equation_correctness already
        // treats 0.1+0.2 = 0.3 as true via a 1e-10 tolerance (verified
        // against the shipped binary in the same session); comparisons use
        // the same tolerance so they don't disagree with `=` at the boundary.
        assert_eq!(eval_math(b"0.1 + 0.2 <= 0.3", &env), Some(1.0));
        assert_eq!(eval_math(b"0.1 + 0.2 != 0.3", &env), Some(0.0));
    }

    #[test]
    fn t59_comparisons_nest_inside_parens_and_calls() {
        let env = TantoEnv::new();
        // parse_expr is the entry point used for parenthesized sub-expressions
        // and function arguments too, so comparisons fall out as first-class
        // sub-expressions for free — not part of the bug, but worth locking in.
        assert_eq!(eval_math(b"(5 > 3) + (2 > 9)", &env), Some(1.0));
        assert_eq!(eval_math(b"max(5 > 3, 0.2)", &env), Some(1.0));
    }

    #[test]
    fn t59_full_consumption_rejects_trailing_garbage() {
        let env = TantoEnv::new();
        // The root cause behind the relational-operator bug was more general
        // than relational operators: eval_math never checked it had consumed
        // every token, so ANY trailing unparseable suffix was silently
        // dropped instead of failing the whole expression. `<`/`>`/`<=`/`>=`
        // are fixed by being real tokens now; this guards the underlying
        // mechanism directly so the same shape of bug can't reappear behind
        // some other future unrecognized character.
        assert_eq!(eval_math(b"5 @ 3", &env), None);
        assert_eq!(eval_math(b"5 3", &env), None); // two numbers, no operator
        assert_eq!(eval_math(b"9.11 <", &env), None); // operator, no RHS
        // Sanity: complete, well-formed input still works exactly as before.
        assert_eq!(eval_math(b"2+3", &env), Some(5.0));
        assert_eq!(eval_math(b"(2+3)*4", &env), Some(20.0));
    }
}
