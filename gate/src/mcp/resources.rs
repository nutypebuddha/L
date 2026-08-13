use crate::inference::json::{JsonValue, stringify};
use crate::kb::facts::KnowledgeBase;

pub fn list_resources(kb: &KnowledgeBase) -> JsonValue {
    let mut resources = Vec::new();

    for fact in &kb.facts {
        let uri = format!("cid://facts/{}", fact.name);
        let name = format!("KB: {}", fact.name);
        let description = if fact.unit.is_empty() {
            fact.source.clone()
        } else {
            format!("{} ({})", fact.source, fact.unit)
        };

        resources.push(JsonValue::Object(vec![
            ("uri".to_string(), JsonValue::Str(uri)),
            ("name".to_string(), JsonValue::Str(name)),
            ("description".to_string(), JsonValue::Str(description)),
            (
                "mimeType".to_string(),
                JsonValue::Str("application/json".to_string()),
            ),
        ]));
    }

    resources.push(JsonValue::Object(vec![
        (
            "uri".to_string(),
            JsonValue::Str("cid://schemas/validation".to_string()),
        ),
        (
            "name".to_string(),
            JsonValue::Str("Validation Result Schema".to_string()),
        ),
        (
            "description".to_string(),
            JsonValue::Str("JSON schema for CID validation results".to_string()),
        ),
        (
            "mimeType".to_string(),
            JsonValue::Str("application/schema+json".to_string()),
        ),
    ]));

    JsonValue::Array(resources)
}

pub fn read_resource(uri: &str, kb: &KnowledgeBase) -> Result<String, String> {
    if let Some(name) = uri.strip_prefix("cid://facts/") {
        match kb.lookup(name) {
            Some(fact) => Ok(stringify(&JsonValue::Object(vec![
                ("name".to_string(), JsonValue::Str(fact.name.clone())),
                ("value".to_string(), JsonValue::Number(fact.value)),
                ("unit".to_string(), JsonValue::Str(fact.unit.clone())),
                ("source".to_string(), JsonValue::Str(fact.source.clone())),
            ]))),
            None => Err(format!("Fact '{}' not found", name)),
        }
    } else if uri == "cid://schemas/validation" {
        Ok(VALIDATION_SCHEMA.to_string())
    } else {
        Err(format!("Unknown resource URI: {}", uri))
    }
}

const VALIDATION_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "validated_text": { "type": "string" },
    "original_text": { "type": "string" },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
    "verdict": { "type": "string", "enum": ["ok", "corrected", "failed", "unevaluable"] },
    "fix_count": { "type": "integer" },
    "gate_scores": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "gate": { "type": "string" },
          "passed": { "type": "boolean" },
          "score": { "type": "number" },
          "details": { "type": "string" }
        }
      }
    }
  }
}"#;

pub fn list_prompts() -> JsonValue {
    JsonValue::Array(vec![
        JsonValue::Object(vec![
            (
                "name".to_string(),
                JsonValue::Str("validate_text".to_string()),
            ),
            (
                "description".to_string(),
                JsonValue::Str("Validate text through CID's gates and return results".to_string()),
            ),
            (
                "arguments".to_string(),
                JsonValue::Array(vec![
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str("text".to_string())),
                        (
                            "description".to_string(),
                            JsonValue::Str("Text to validate".to_string()),
                        ),
                        ("required".to_string(), JsonValue::Bool(true)),
                    ]),
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str("context".to_string())),
                        (
                            "description".to_string(),
                            JsonValue::Str(
                                "Domain context (math, fact, logic, english)".to_string(),
                            ),
                        ),
                        ("required".to_string(), JsonValue::Bool(false)),
                    ]),
                ]),
            ),
        ]),
        JsonValue::Object(vec![
            ("name".to_string(), JsonValue::Str("fix_text".to_string())),
            (
                "description".to_string(),
                JsonValue::Str("Auto-fix errors in text (math, typos, consistency)".to_string()),
            ),
            (
                "arguments".to_string(),
                JsonValue::Array(vec![
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str("text".to_string())),
                        (
                            "description".to_string(),
                            JsonValue::Str("Text to fix".to_string()),
                        ),
                        ("required".to_string(), JsonValue::Bool(true)),
                    ]),
                    JsonValue::Object(vec![
                        ("name".to_string(), JsonValue::Str("context".to_string())),
                        (
                            "description".to_string(),
                            JsonValue::Str(
                                "Domain context (math, javascript, english)".to_string(),
                            ),
                        ),
                        ("required".to_string(), JsonValue::Bool(false)),
                    ]),
                ]),
            ),
        ]),
        JsonValue::Object(vec![
            (
                "name".to_string(),
                JsonValue::Str("analyze_reasoning".to_string()),
            ),
            (
                "description".to_string(),
                JsonValue::Str(
                    "Analyze text for logical fallacies and cognitive biases".to_string(),
                ),
            ),
            (
                "arguments".to_string(),
                JsonValue::Array(vec![JsonValue::Object(vec![
                    ("name".to_string(), JsonValue::Str("text".to_string())),
                    (
                        "description".to_string(),
                        JsonValue::Str("Text to analyze".to_string()),
                    ),
                    ("required".to_string(), JsonValue::Bool(true)),
                ])]),
            ),
        ]),
    ])
}

pub fn get_prompt(name: &str, arguments: &JsonValue) -> Result<JsonValue, String> {
    let args = arguments.as_object().ok_or("arguments must be an object")?;

    match name {
        "validate_text" => {
            let text = args
                .iter()
                .find(|(k, _)| k == "text")
                .and_then(|(_, v)| v.as_str())
                .ok_or("Missing required argument: text")?;
            let context = args
                .iter()
                .find(|(k, _)| k == "context")
                .and_then(|(_, v)| v.as_str())
                .unwrap_or("general");

            Ok(JsonValue::Object(vec![
                (
                    "description".to_string(),
                    JsonValue::Str(format!("Validate '{}' in {} context", text, context)),
                ),
                (
                    "messages".to_string(),
                    JsonValue::Array(vec![JsonValue::Object(vec![
                        ("role".to_string(), JsonValue::Str("user".to_string())),
                        (
                            "content".to_string(),
                            JsonValue::Str(format!(
                                "Use the cid_validate tool to validate this text: \"{}\" with context \"{}\"",
                                text, context
                            )),
                        ),
                    ])]),
                ),
            ]))
        }
        "fix_text" => {
            let text = args
                .iter()
                .find(|(k, _)| k == "text")
                .and_then(|(_, v)| v.as_str())
                .ok_or("Missing required argument: text")?;
            let context = args
                .iter()
                .find(|(k, _)| k == "context")
                .and_then(|(_, v)| v.as_str())
                .unwrap_or("general");

            Ok(JsonValue::Object(vec![
                (
                    "description".to_string(),
                    JsonValue::Str(format!("Fix errors in '{}'", text)),
                ),
                (
                    "messages".to_string(),
                    JsonValue::Array(vec![JsonValue::Object(vec![
                        ("role".to_string(), JsonValue::Str("user".to_string())),
                        (
                            "content".to_string(),
                            JsonValue::Str(format!(
                                "Use the cid_fix tool to correct errors in: \"{}\" with context \"{}\"",
                                text, context
                            )),
                        ),
                    ])]),
                ),
            ]))
        }
        "analyze_reasoning" => {
            let text = args
                .iter()
                .find(|(k, _)| k == "text")
                .and_then(|(_, v)| v.as_str())
                .ok_or("Missing required argument: text")?;

            Ok(JsonValue::Object(vec![
                (
                    "description".to_string(),
                    JsonValue::Str("Analyze reasoning in text".to_string()),
                ),
                (
                    "messages".to_string(),
                    JsonValue::Array(vec![JsonValue::Object(vec![
                        ("role".to_string(), JsonValue::Str("user".to_string())),
                        (
                            "content".to_string(),
                            JsonValue::Str(format!(
                                "Use cid_detect_fallacies and cid_detect_biases to analyze: \"{}\"",
                                text
                            )),
                        ),
                    ])]),
                ),
            ]))
        }
        _ => Err(format!("Unknown prompt: {}", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::json::parse_json;

    #[test]
    fn test_list_resources() {
        let kb = KnowledgeBase::new();
        let resources = list_resources(&kb);
        let arr = resources.as_array().unwrap();
        assert!(arr.len() > 10);
    }

    #[test]
    fn test_read_resource_pi() {
        let kb = KnowledgeBase::new();
        let result = read_resource("cid://facts/pi", &kb).unwrap();
        assert!(result.contains("pi"));
        assert!(result.contains("3.14159"));
    }

    #[test]
    fn test_read_resource_not_found() {
        let kb = KnowledgeBase::new();
        let result = read_resource("cid://facts/nonexistent", &kb);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_prompts() {
        let prompts = list_prompts();
        let arr = prompts.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_get_prompt_validate() {
        let args = parse_json(r#"{"text":"2+2=5","context":"math"}"#).unwrap();
        let prompt = get_prompt("validate_text", &args).unwrap();
        assert!(prompt.get_str("description").unwrap().contains("2+2=5"));
    }

    #[test]
    fn test_get_prompt_unknown() {
        let args = parse_json(r#"{}"#).unwrap();
        let result = get_prompt("unknown", &args);
        assert!(result.is_err());
    }
}
