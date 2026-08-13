/// Pure function: Parse natural language query into structured intent.
pub fn parse_query_intent(query: &str) -> &'static str {
    let normalized = query.to_lowercase();
    if normalized.starts_with("what") || normalized.starts_with("how") {
        "question"
    } else if normalized.starts_with("compute") || normalized.starts_with("calculate") {
        "computation"
    } else if normalized.starts_with("validate") || normalized.starts_with("check") {
        "validation"
    } else if normalized.starts_with("find") || normalized.starts_with("search") {
        "search"
    } else {
        "general"
    }
}

/// Pure function: Extract numerical values from query text.
pub fn extract_numerical_values(query: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let bytes = query.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            // Only treat this as a scalar if it starts a token (the preceding
            // char is not an identifier character). This stops identifiers like
            // `mass1` from yielding the number 1.
            let mid_identifier = i > 0 && bytes[i - 1].is_ascii_alphanumeric();
            let start = i;
            i += 1;
            let mut saw_dot = false;
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_digit() {
                    i += 1;
                } else if b == b'.' && !saw_dot {
                    saw_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            if !mid_identifier
                && let Ok(st) = std::str::from_utf8(&bytes[start..i])
                && let Ok(s) = st.parse::<f64>()
            {
                out.push(s);
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Pure function: Determine domain from query keywords.
pub fn determine_query_domain(query: &str) -> &'static str {
    let normalized = query.to_lowercase();
    if normalized.contains("graha") || normalized.contains("planet") {
        "astrology"
    } else if normalized.contains("formula") || normalized.contains("equation") {
        "formula"
    } else if normalized.contains("entity") || normalized.contains("object") {
        "entity"
    } else if normalized.contains("test") || normalized.contains("validate") {
        "validation"
    } else {
        "general"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_query_intent_basic() {
        assert_eq!(parse_query_intent("what is surya"), "question");
        assert_eq!(parse_query_intent("compute the value"), "computation");
        assert_eq!(parse_query_intent("validate this formula"), "validation");
        assert_eq!(parse_query_intent("find grahas"), "search");
        assert_eq!(parse_query_intent("tell me something"), "general");
    }

    #[test]
    // `3.14` here is a parsed decimal extracted from input text, not the
    // transcendental constant — clippy's approx_constant lint does not apply.
    #[allow(clippy::approx_constant)]
    fn extract_numerical_values_basic() {
        assert_eq!(
            extract_numerical_values("compute 42 and 3.14"),
            vec![42.0, 3.14]
        );
        assert!(extract_numerical_values("no numbers here").is_empty());
    }

    #[test]
    fn determine_query_domain_basic() {
        assert_eq!(
            determine_query_domain("tell me about surya graha"),
            "astrology"
        );
        assert_eq!(determine_query_domain("compute this formula"), "formula");
        assert_eq!(determine_query_domain("find the entity"), "entity");
        assert_eq!(determine_query_domain("something else"), "general");
    }
}
