use super::math;
use super::parser;
use super::TantoEnv;

pub fn evaluate_pipeline(line: &str, env: &TantoEnv) -> Option<f64> {
    let (segments, seg_count) = split_pipeline(line.as_bytes());
    if seg_count == 0 {
        return None;
    }
    if seg_count == 1 {
        return eval_segment(segments[0], env, None);
    }

    let mut prev: Option<f64> = None;
    for seg in &segments[..seg_count] {
        let val = eval_segment(seg, env, prev)?;
        prev = Some(val);
    }
    prev
}

fn eval_segment(seg: &[u8], env: &TantoEnv, prev: Option<f64>) -> Option<f64> {
    let seg = trim(seg);
    if seg.is_empty() {
        return None;
    }

    if seg == b"_" {
        return prev;
    }

    if let Some(v) = parser::eval_math(seg, env) {
        return Some(v);
    }

    let tokens = tokenize(seg);
    if tokens.is_empty() {
        return None;
    }
    let op = std::str::from_utf8(tokens[0]).ok()?;
    let mut args = [0.0f64; 4];
    let mut argc = 0;
    for tok in &tokens[1..] {
        if *tok == b"_" {
            args[argc] = prev.unwrap_or(0.0);
        } else {
            args[argc] = parse_arg_value(tok, env)?;
        }
        argc += 1;
    }

    math::compute_named(op, &args[..argc])
}

fn parse_arg_value(tok: &[u8], env: &TantoEnv) -> Option<f64> {
    if tok == b"_" {
        return None;
    }
    let name = std::str::from_utf8(tok).ok()?;
    if let Some(v) = env.get(name) {
        return Some(*v);
    }
    name.parse::<f64>().ok()
}

fn tokenize(line: &[u8]) -> Vec<&[u8]> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < line.len() {
        while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
            i += 1;
        }
        if i >= line.len() {
            break;
        }
        let start = i;
        while i < line.len() && line[i] != b' ' && line[i] != b'\t' {
            i += 1;
        }
        tokens.push(&line[start..i]);
    }
    tokens
}

fn split_pipeline(line: &[u8]) -> ([&[u8]; 8], usize) {
    let mut segments = [&b""[..]; 8];
    let mut count = 0;
    let mut start = 0;
    let mut depth = 0;

    for i in 0..line.len() {
        match line[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'|' if depth == 0 => {
                if count < 8 {
                    segments[count] = &line[start..i];
                    count += 1;
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if count < 8 {
        segments[count] = &line[start..];
        count += 1;
    }
    (segments, count)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_pipeline_simple() {
        let env = TantoEnv::new();
        assert_eq!(evaluate_pipeline("add 2 3", &env), Some(5.0));
    }

    #[test]
    fn test_evaluate_pipeline_chain() {
        let env = TantoEnv::new();
        assert_eq!(evaluate_pipeline("div 1 3 | mul 6 _", &env), Some(2.0));
    }

    #[test]
    fn test_evaluate_pipeline_multi() {
        let env = TantoEnv::new();
        assert_eq!(evaluate_pipeline("add 2 3 | mul 4 _", &env), Some(20.0));
    }

    #[test]
    fn test_evaluate_pipeline_expr() {
        let env = TantoEnv::new();
        assert_eq!(evaluate_pipeline("2+3", &env), Some(5.0));
    }
}
