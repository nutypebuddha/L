pub fn preprocess_line(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(b"whats ") || input[i..].starts_with(b"What's ") {
            i += 6;
        } else if input[i..].starts_with(b"what is ") || input[i..].starts_with(b"What is ") {
            i += 8;
        } else if input[i..].starts_with(b"calculate ") || input[i..].starts_with(b"Calculate ") {
            i += 10;
        } else if input[i..].starts_with(b"compute ") || input[i..].starts_with(b"Compute ") {
            i += 8;
        } else if input[i..].starts_with(b"how much is ") || input[i..].starts_with(b"How much is ")
        {
            i += 12;
        } else if input[i..].starts_with(b"solve ") || input[i..].starts_with(b"Solve ") {
            i += 6;
        } else if input[i..].starts_with(b"eval ") || input[i..].starts_with(b"Eval ") {
            i += 5;
        } else {
            output.push(input[i]);
            i += 1;
        }
    }

    let s = String::from_utf8_lossy(&output).to_string();
    if let Some(result) = evaluate_percentage(&s) {
        return result.to_string().into_bytes();
    }

    output
}

pub fn parse_percentage(input: &str) -> Option<(f64, f64)> {
    let input = input.trim().to_lowercase();
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() >= 3 && parts[1] == "of" {
        let pct = parts[0].trim_end_matches('%').parse::<f64>().ok()?;
        let val = parts[2].parse::<f64>().ok()?;
        Some((pct, val))
    } else {
        None
    }
}

pub fn evaluate_percentage(input: &str) -> Option<f64> {
    let (pct, val) = parse_percentage(input)?;
    Some(val * pct / 100.0)
}

pub fn preprocess_math_input(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    if let Some(result) = evaluate_percentage(input) {
        return Some(result.to_string());
    }

    let processed = preprocess_line(input.as_bytes());
    let s = String::from_utf8(processed).ok()?;
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_line() {
        assert_eq!(preprocess_line(b"whats 2+3"), b"2+3");
        assert_eq!(preprocess_line(b"what is 2+3"), b"2+3");
        assert_eq!(preprocess_line(b"calculate 2+3"), b"2+3");
        assert_eq!(preprocess_line(b"2+3"), b"2+3");
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("15% of 240"), Some((15.0, 240.0)));
        assert_eq!(parse_percentage("10% of 250"), Some((10.0, 250.0)));
    }

    #[test]
    fn test_evaluate_percentage() {
        assert_eq!(evaluate_percentage("15% of 240"), Some(36.0));
        assert_eq!(evaluate_percentage("10% of 250"), Some(25.0));
    }

    #[test]
    fn test_preprocess_math_input() {
        assert_eq!(preprocess_math_input("whats 2+3"), Some("2+3".to_string()));
        assert_eq!(preprocess_math_input("15% of 240"), Some("36".to_string()));
    }
}
