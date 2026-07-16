use crate::compute;

pub fn evaluate_pipeline(line: &str) -> Option<f64> {
    let env = compute::create_env();
    compute::evaluate_pipeline(line, &env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_pipeline() {
        assert_eq!(evaluate_pipeline("div 1 3 | mul 6 _"), Some(2.0));
    }

    #[test]
    fn test_evaluate_pipeline_simple() {
        assert_eq!(evaluate_pipeline("add 2 3"), Some(5.0));
    }
}
