// Tanto pipeline — pipeline operator: "div 1 3 | mul 6 _"
// Adapted for CID

use crate::tanto::math;
use crate::tanto::parser;
use crate::tanto::TantoEnv;

/// Evaluate a pipeline expression
/// Segments separated by |, _ refers to previous result
/// Example: "div 1 3 | mul 6 _" evaluates to 2.0
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

    // Check for pipeline placeholder _
    if seg == b"_" {
        return prev;
    }

    // Try expression first (handles _, variables, constants)
    if let Some(v) = parser::eval_math(seg, env) {
        return Some(v);
    }

    // Try op format: "op arg1 arg2"
    let tokens = tokenize(seg);
    if tokens.is_empty() {
        return None;
    }
    let op = tokens[0];
    let mut args = [0.0f64; 4];
    for (argc, tok) in tokens[1..].iter().enumerate() {
        if *tok == b"_" {
            args[argc] = prev.unwrap_or(0.0);
        } else {
            args[argc] = parse_arg_value(tok, env)?;
        }
    }

    math::compute_named(op, &args[..tokens.len() - 1])
}

fn parse_arg_value(tok: &[u8], env: &TantoEnv) -> Option<f64> {
    if tok == b"_" {
        return None;
    }
    let name = std::str::from_utf8(tok).ok()?;
    if let Some(v) = env.get(name) {
        return Some(v);
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
