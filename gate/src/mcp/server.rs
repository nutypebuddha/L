use std::io::{self, BufRead, Write};

use super::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::resources;
use super::tools;
use crate::inference::json::JsonValue;
use crate::inference::InferenceEngine;

pub fn run() {
    let mut engine = InferenceEngine::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = handle_message(trimmed, &mut engine);
        if let Some(json) = response {
            let _ = stdout.write_all(json.as_bytes());
            let _ = stdout.write_all(b"\n");
            let _ = stdout.flush();
        }
    }
}

pub fn handle_message(input: &str, engine: &mut InferenceEngine) -> Option<String> {
    let request = match JsonRpcRequest::parse(input) {
        Some(r) => r,
        None => {
            return Some(JsonRpcError::parse_error("Invalid JSON-RPC").to_json());
        }
    };

    match request.method.as_str() {
        "initialize" => Some(handle_initialize(&request)),
        "notifications/initialized" => None,
        "ping" => {
            Some(JsonRpcResponse::success(request.id.clone(), JsonValue::Object(vec![])).to_json())
        }
        "tools/list" => Some(handle_tools_list(&request)),
        "tools/call" => Some(handle_tools_call(&request, engine)),
        "resources/list" => Some(handle_resources_list(&request, engine)),
        "resources/read" => Some(handle_resources_read(&request, engine)),
        "prompts/list" => Some(handle_prompts_list(&request)),
        "prompts/get" => Some(handle_prompts_get(&request)),
        _ => Some(JsonRpcError::method_not_found(request.id, &request.method).to_json()),
    }
}

fn handle_initialize(request: &JsonRpcRequest) -> String {
    let result = JsonValue::Object(vec![
        (
            "protocolVersion".to_string(),
            JsonValue::Str("2025-03-26".to_string()),
        ),
        (
            "capabilities".to_string(),
            JsonValue::Object(vec![
                ("tools".to_string(), JsonValue::Object(vec![])),
                ("resources".to_string(), JsonValue::Object(vec![])),
                ("prompts".to_string(), JsonValue::Object(vec![])),
            ]),
        ),
        (
            "serverInfo".to_string(),
            JsonValue::Object(vec![
                ("name".to_string(), JsonValue::Str("cid".to_string())),
                ("version".to_string(), JsonValue::Str("0.8.0".to_string())),
            ]),
        ),
    ]);
    JsonRpcResponse::success(request.id.clone(), result).to_json()
}

fn handle_tools_list(request: &JsonRpcRequest) -> String {
    let tools = tools::list_tools();
    let result = JsonValue::Object(vec![("tools".to_string(), tools)]);
    JsonRpcResponse::success(request.id.clone(), result).to_json()
}

fn handle_tools_call(request: &JsonRpcRequest, engine: &mut InferenceEngine) -> String {
    let params = request.params.as_object();
    let name = params
        .and_then(|p| p.iter().find(|(k, _)| k == "name"))
        .and_then(|(_, v)| v.as_str());
    let arguments = params
        .and_then(|p| p.iter().find(|(k, _)| k == "arguments"))
        .map(|(_, v)| v.clone())
        .unwrap_or(JsonValue::Object(vec![]));

    let name = match name {
        Some(n) => n,
        None => {
            return JsonRpcError::invalid_params(request.id.clone(), "Missing tool name").to_json();
        }
    };

    match tools::call_tool(name, &arguments, engine) {
        Ok(text) => super::protocol::tool_result(request.id.clone(), &text),
        Err(text) => super::protocol::tool_error(request.id.clone(), &text),
    }
}

fn handle_resources_list(request: &JsonRpcRequest, engine: &InferenceEngine) -> String {
    let resources_list = resources::list_resources(engine.kb());
    let result = JsonValue::Object(vec![("resources".to_string(), resources_list)]);
    JsonRpcResponse::success(request.id.clone(), result).to_json()
}

fn handle_resources_read(request: &JsonRpcRequest, engine: &InferenceEngine) -> String {
    let params = request.params.as_object();
    let uri = params
        .and_then(|p| p.iter().find(|(k, _)| k == "uri"))
        .and_then(|(_, v)| v.as_str());

    let uri = match uri {
        Some(u) => u,
        None => {
            return JsonRpcError::invalid_params(request.id.clone(), "Missing resource URI")
                .to_json();
        }
    };

    match resources::read_resource(uri, engine.kb()) {
        Ok(content) => {
            let result = JsonValue::Object(vec![(
                "contents".to_string(),
                JsonValue::Array(vec![JsonValue::Object(vec![
                    ("uri".to_string(), JsonValue::Str(uri.to_string())),
                    (
                        "mimeType".to_string(),
                        JsonValue::Str("application/json".to_string()),
                    ),
                    ("text".to_string(), JsonValue::Str(content)),
                ])]),
            )]);
            JsonRpcResponse::success(request.id.clone(), result).to_json()
        }
        Err(e) => JsonRpcError::invalid_params(request.id.clone(), &e).to_json(),
    }
}

fn handle_prompts_list(request: &JsonRpcRequest) -> String {
    let prompts = resources::list_prompts();
    let result = JsonValue::Object(vec![("prompts".to_string(), prompts)]);
    JsonRpcResponse::success(request.id.clone(), result).to_json()
}

fn handle_prompts_get(request: &JsonRpcRequest) -> String {
    let params = request.params.as_object();
    let name = params
        .and_then(|p| p.iter().find(|(k, _)| k == "name"))
        .and_then(|(_, v)| v.as_str());
    let arguments = params
        .and_then(|p| p.iter().find(|(k, _)| k == "arguments"))
        .map(|(_, v)| v.clone())
        .unwrap_or(JsonValue::Object(vec![]));

    let name = match name {
        Some(n) => n,
        None => {
            return JsonRpcError::invalid_params(request.id.clone(), "Missing prompt name")
                .to_json();
        }
    };

    match resources::get_prompt(name, &arguments) {
        Ok(result) => JsonRpcResponse::success(request.id.clone(), result).to_json(),
        Err(e) => JsonRpcError::invalid_params(request.id.clone(), &e).to_json(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let input = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("protocolVersion"));
        assert!(resp.contains("cid"));
        assert!(resp.contains("0.8.0"));
    }

    #[test]
    fn test_tools_list() {
        let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("cid_validate"));
        assert!(resp.contains("cid_fix"));
        assert!(resp.contains("cid_lookup"));
        assert!(resp.contains("cid_search"));
    }

    #[test]
    fn test_tools_call_validate() {
        let input = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"cid_validate","arguments":{"text":"2 + 3 = 6","context":"math"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("validated_text"));
        assert!(resp.contains("2 + 3 = 5"));
        assert!(resp.contains("fix_count"));
    }

    #[test]
    fn test_tools_call_fix() {
        let input = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"cid_fix","arguments":{"text":"hte cat sat on teh mat","context":"english"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("the cat sat on the mat"));
        assert!(resp.contains("fixes"));
    }

    #[test]
    fn test_tools_call_lookup() {
        let input = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"cid_lookup","arguments":{"name":"pi"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("3.14159"));
        assert!(resp.contains("math constant"));
    }

    #[test]
    fn test_tools_call_search() {
        let input = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"cid_search","arguments":{"query":"speed"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("speed_sound"));
        assert!(resp.contains("results"));
    }

    #[test]
    fn test_method_not_found() {
        let input = r#"{"jsonrpc":"2.0","id":7,"method":"nonexistent","params":{}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("-32601"));
    }

    #[test]
    fn test_ping() {
        let input = r#"{"jsonrpc":"2.0","id":8,"method":"ping","params":{}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("8"));
    }

    #[test]
    fn test_invalid_json() {
        let resp = handle_message("not json", &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("-32700"));
    }

    #[test]
    fn test_notifications_initialized_no_response() {
        let input = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let resp = handle_message(input, &mut InferenceEngine::new());
        assert!(resp.is_none());
    }

    #[test]
    fn test_tools_call_detect_fallacies() {
        let input = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"cid_detect_fallacies","arguments":{"text":"You are stupid and everyone knows it"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("ad_hominem"));
        assert!(resp.contains("bandwagon"));
    }

    #[test]
    fn test_tools_call_sanity_check() {
        let input = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"cid_sanity_check","arguments":{"value":60.0,"category":"speed_mph"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("in_range"));
        assert!(resp.contains("in_typical"));
    }

    #[test]
    fn test_tools_call_detect_biases() {
        let input = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"cid_detect_biases","arguments":{"text":"I knew it all along and it proves my point and everyone thinks so"}}}"#;
        let resp = handle_message(input, &mut InferenceEngine::new()).unwrap();
        assert!(resp.contains("hindsight"));
        assert!(resp.contains("confirmation"));
        assert!(resp.contains("bandwagon"));
    }
}
