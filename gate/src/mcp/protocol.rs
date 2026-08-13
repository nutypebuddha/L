use crate::inference::json::{parse_json, JsonValue};

#[derive(Debug, Clone)]
pub struct JsonRpcRequest {
    pub id: Option<JsonValue>,
    pub method: String,
    pub params: JsonValue,
}

#[derive(Debug, Clone)]
pub struct JsonRpcResponse {
    pub id: Option<JsonValue>,
    pub result: JsonValue,
}

#[derive(Debug, Clone)]
pub struct JsonRpcError {
    pub id: Option<JsonValue>,
    pub code: i64,
    pub message: String,
    pub data: Option<JsonValue>,
}

impl JsonRpcRequest {
    pub fn parse(input: &str) -> Option<Self> {
        let json = parse_json(input).ok()?;
        let obj = json.as_object()?;

        let jsonrpc = obj.iter().find(|(k, _)| k == "jsonrpc")?.1.as_str()?;
        if jsonrpc != "2.0" {
            return None;
        }

        let method = obj
            .iter()
            .find(|(k, _)| k == "method")?
            .1
            .as_str()?
            .to_string();
        let id = obj.iter().find(|(k, _)| k == "id").map(|(_, v)| v.clone());
        let params = obj
            .iter()
            .find(|(k, _)| k == "params")
            .map(|(_, v)| v.clone())
            .unwrap_or(JsonValue::Object(vec![]));

        Some(JsonRpcRequest { id, method, params })
    }
}

impl JsonRpcResponse {
    pub fn success(id: Option<JsonValue>, result: JsonValue) -> Self {
        JsonRpcResponse { id, result }
    }

    pub fn to_json(&self) -> String {
        let mut fields = vec![
            ("jsonrpc".to_string(), JsonValue::Str("2.0".to_string())),
            ("result".to_string(), self.result.clone()),
        ];
        if let Some(ref id) = self.id {
            fields.insert(1, ("id".to_string(), id.clone()));
        } else {
            fields.insert(1, ("id".to_string(), JsonValue::Null));
        }
        crate::inference::json::stringify(&JsonValue::Object(fields))
    }
}

impl JsonRpcError {
    pub fn new(id: Option<JsonValue>, code: i64, message: &str) -> Self {
        JsonRpcError {
            id,
            code,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn method_not_found(id: Option<JsonValue>, method: &str) -> Self {
        Self::new(id, -32601, &format!("Method not found: {}", method))
    }

    pub fn invalid_params(id: Option<JsonValue>, msg: &str) -> Self {
        Self::new(id, -32602, msg)
    }

    pub fn parse_error(msg: &str) -> Self {
        Self::new(None, -32700, msg)
    }

    pub fn internal_error(id: Option<JsonValue>, msg: &str) -> Self {
        Self::new(id, -32603, msg)
    }

    pub fn to_json(&self) -> String {
        let mut error_fields = vec![
            ("code".to_string(), JsonValue::Number(self.code as f64)),
            ("message".to_string(), JsonValue::Str(self.message.clone())),
        ];
        if let Some(ref data) = self.data {
            error_fields.push(("data".to_string(), data.clone()));
        }

        let mut fields = vec![
            ("jsonrpc".to_string(), JsonValue::Str("2.0".to_string())),
            ("error".to_string(), JsonValue::Object(error_fields)),
        ];
        if let Some(ref id) = self.id {
            fields.insert(1, ("id".to_string(), id.clone()));
        } else {
            fields.insert(1, ("id".to_string(), JsonValue::Null));
        }
        crate::inference::json::stringify(&JsonValue::Object(fields))
    }
}

pub fn content_text(text: &str) -> JsonValue {
    JsonValue::Object(vec![
        ("type".to_string(), JsonValue::Str("text".to_string())),
        ("text".to_string(), JsonValue::Str(text.to_string())),
    ])
}

pub fn tool_result(id: Option<JsonValue>, text: &str) -> String {
    let resp = JsonRpcResponse::success(
        id,
        JsonValue::Object(vec![
            (
                "content".to_string(),
                JsonValue::Array(vec![content_text(text)]),
            ),
            ("isError".to_string(), JsonValue::Bool(false)),
        ]),
    );
    resp.to_json()
}

pub fn tool_error(id: Option<JsonValue>, text: &str) -> String {
    let resp = JsonRpcResponse::success(
        id,
        JsonValue::Object(vec![
            (
                "content".to_string(),
                JsonValue::Array(vec![content_text(text)]),
            ),
            ("isError".to_string(), JsonValue::Bool(true)),
        ]),
    );
    resp.to_json()
}
