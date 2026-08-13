use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use super::InferenceEngine;
use super::json::{JsonValue, parse_json, stringify};
use super::request::{ProxyConfig, ValidationRequest};
use super::result::{CidError, CidResult};
use super::stream::StreamValidator;

#[cfg(feature = "proxy")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    Generic,
}

#[cfg(feature = "proxy")]
impl Provider {
    fn detect(endpoint: &str) -> Self {
        let lower = endpoint.to_lowercase();
        if lower.contains("anthropic") || lower.contains("/messages") {
            Provider::Anthropic
        } else if lower.contains("generativelanguage") || lower.contains("gemini") {
            Provider::Gemini
        } else if lower.contains("openai") || lower.contains("/chat/completions") {
            Provider::OpenAI
        } else {
            Provider::Generic
        }
    }
}

pub struct ProxyServer {
    config: ProxyConfig,
    calibration: std::sync::Mutex<crate::gates::confidence::CalibrationData>,
}

impl ProxyServer {
    pub fn new(config: ProxyConfig) -> Self {
        ProxyServer {
            config,
            calibration: std::sync::Mutex::new(crate::gates::confidence::CalibrationData::new()),
        }
    }

    pub fn run(&self) -> CidResult<()> {
        let listener = TcpListener::bind(&self.config.listen_addr)
            .map_err(|e| CidError::IoError(format!("Failed to bind: {}", e)))?;

        eprintln!("CID Proxy listening on {}", self.config.listen_addr);
        eprintln!("LLM endpoint: {}", self.config.llm_endpoint);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let config = self.config.clone();
                    let calibration =
                        std::sync::Mutex::new(self.calibration.lock().unwrap().clone());
                    thread::spawn(move || {
                        let server = ProxyServer {
                            config,
                            calibration,
                        };
                        if let Err(e) = server.handle_connection(stream) {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) -> CidResult<()> {
        let addr = stream
            .peer_addr()
            .map_err(|e| CidError::IoError(format!("Failed to get peer address: {}", e)))?;

        eprintln!("New connection from {}", addr);

        let mut reader = BufReader::new(
            stream
                .try_clone()
                .map_err(|e| CidError::IoError(format!("Failed to clone stream: {}", e)))?,
        );

        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .map_err(|e| CidError::IoError(format!("Failed to read request line: {}", e)))?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            Self::send_error(&mut stream, 400, "Bad Request")?;
            return Ok(());
        }

        let method = parts[0];
        let path = parts[1];

        let mut _headers = Vec::new();
        let mut content_length = 0;

        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| CidError::IoError(format!("Failed to read header: {}", e)))?;

            let line = line.trim().to_string();
            if line.is_empty() {
                break;
            }

            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len) = line.split(':').nth(1) {
                    content_length = len.trim().parse::<usize>().unwrap_or(0);
                }
            }

            _headers.push(line);
        }

        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader
                .read_exact(&mut body)
                .map_err(|e| CidError::IoError(format!("Failed to read body: {}", e)))?;
        }

        let body_str = String::from_utf8_lossy(&body).to_string();

        match (method, path) {
            ("POST", "/v1/validate") => {
                self.handle_validate(&mut stream, &body_str)?;
            }
            ("POST", "/v1/proxy") => {
                self.handle_proxy(&mut stream, &body_str)?;
            }
            ("POST", "/v1/stream") => {
                self.handle_stream(&mut stream, &body_str)?;
            }
            ("POST", "/v1/feedback") => {
                self.handle_feedback(&mut stream, &body_str)?;
            }
            ("GET", "/v1/health") => {
                self.handle_health(&mut stream)?;
            }
            ("GET", "/v1/stats") => {
                self.handle_stats(&mut stream)?;
            }
            _ => {
                Self::send_error(&mut stream, 404, "Not Found")?;
            }
        }

        Ok(())
    }

    fn handle_validate(&self, stream: &mut TcpStream, body: &str) -> CidResult<()> {
        let json =
            parse_json(body).map_err(|e| CidError::ParseError(format!("Invalid JSON: {}", e)))?;

        let text = json
            .get_str("text")
            .ok_or_else(|| CidError::InvalidInput("Missing 'text' field".to_string()))?;
        let context = json.get_str("context").unwrap_or("");
        let domain = json.get_str("domain");

        let request = if let Some(d) = domain {
            ValidationRequest::new(text, context).with_domain(d)
        } else {
            ValidationRequest::new(text, context)
        };

        let mut engine = InferenceEngine::new();
        let result = engine
            .validate(request)
            .map_err(|e| CidError::LlmError(format!("Validation failed: {}", e)))?;

        let validated_text = result.validated_text.clone();
        let original_text = result.original_text.clone();
        let confidence = result.confidence;
        let verdict = result.verdict().as_str().to_string();
        let fix_count = result.fix_count() as f64;

        let response = JsonValue::Object(vec![
            ("validated_text".to_string(), JsonValue::Str(validated_text)),
            ("original_text".to_string(), JsonValue::Str(original_text)),
            ("confidence".to_string(), JsonValue::Number(confidence)),
            ("verdict".to_string(), JsonValue::Str(verdict)),
            ("fix_count".to_string(), JsonValue::Number(fix_count)),
        ]);

        self.send_json(stream, 200, &response)
    }

    fn handle_proxy(&self, stream: &mut TcpStream, body: &str) -> CidResult<()> {
        let json =
            parse_json(body).map_err(|e| CidError::ParseError(format!("Invalid JSON: {}", e)))?;

        let prompt = json
            .get_str("prompt")
            .ok_or_else(|| CidError::InvalidInput("Missing 'prompt' field".to_string()))?;
        let model = json.get_str("model").unwrap_or("gpt-4");
        let max_tokens = json.get_number("max_tokens").unwrap_or(1000.0) as u32;

        let llm_response = self.call_llm(prompt, model, max_tokens)?;

        let request = ValidationRequest::new(&llm_response, prompt);
        let mut engine = InferenceEngine::new();
        let result = engine
            .validate(request)
            .map_err(|e| CidError::LlmError(format!("Validation failed: {}", e)))?;

        let verdict = result.verdict().as_str().to_string();
        let response = JsonValue::Object(vec![
            (
                "response".to_string(),
                JsonValue::Str(result.validated_text),
            ),
            (
                "confidence".to_string(),
                JsonValue::Number(result.confidence),
            ),
            ("verdict".to_string(), JsonValue::Str(verdict)),
            ("model".to_string(), JsonValue::Str(model.to_string())),
        ]);

        self.send_json(stream, 200, &response)
    }

    fn handle_health(&self, stream: &mut TcpStream) -> CidResult<()> {
        let response = JsonValue::Object(vec![
            ("status".to_string(), JsonValue::Str("ok".to_string())),
            ("version".to_string(), JsonValue::Str("0.8.0".to_string())),
        ]);

        self.send_json(stream, 200, &response)
    }

    fn handle_stream(&self, stream: &mut TcpStream, body: &str) -> CidResult<()> {
        let json =
            parse_json(body).map_err(|e| CidError::ParseError(format!("Invalid JSON: {}", e)))?;

        let tokens_raw = json
            .get_str("tokens")
            .ok_or_else(|| CidError::InvalidInput("Missing 'tokens' field".to_string()))?;
        let context = json.get_str("context").unwrap_or("general");

        let tokens: Vec<String> = tokens_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut validator = StreamValidator::new();
        let events = validator.validate_stream(&tokens, context);

        let sse_header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
        stream
            .write_all(sse_header.as_bytes())
            .map_err(|e| CidError::IoError(format!("Failed to write SSE header: {}", e)))?;

        for event in &events {
            let sse_data = event.to_sse();
            stream
                .write_all(sse_data.as_bytes())
                .map_err(|e| CidError::IoError(format!("Failed to write SSE event: {}", e)))?;
            stream
                .flush()
                .map_err(|e| CidError::IoError(format!("Failed to flush SSE: {}", e)))?;
        }

        Ok(())
    }

    fn handle_feedback(&self, stream: &mut TcpStream, body: &str) -> CidResult<()> {
        let json =
            parse_json(body).map_err(|e| CidError::ParseError(format!("Invalid JSON: {}", e)))?;

        let raw_score = json
            .get_number("raw_score")
            .ok_or_else(|| CidError::InvalidInput("Missing 'raw_score' field".to_string()))?;
        let is_correct = json
            .get_bool("is_correct")
            .ok_or_else(|| CidError::InvalidInput("Missing 'is_correct' field".to_string()))?;

        if let Ok(mut cal) = self.calibration.lock() {
            cal.add(raw_score, is_correct);
        }

        let response = JsonValue::Object(vec![
            ("status".to_string(), JsonValue::Str("ok".to_string())),
            (
                "message".to_string(),
                JsonValue::Str("Feedback recorded".to_string()),
            ),
        ]);

        self.send_json(stream, 200, &response)
    }

    fn handle_stats(&self, stream: &mut TcpStream) -> CidResult<()> {
        let response = JsonValue::Object(vec![
            ("status".to_string(), JsonValue::Str("ok".to_string())),
            ("uptime".to_string(), JsonValue::Number(0.0)),
        ]);

        self.send_json(stream, 200, &response)
    }

    fn call_llm(&self, prompt: &str, model: &str, max_tokens: u32) -> CidResult<String> {
        #[cfg(feature = "proxy")]
        {
            use std::time::Duration;

            let agent = ureq::Agent::new_with_config(ureq::config::Config {
                timeout_global: Some(Duration::from_millis(self.config.timeout_ms)),
                ..Default::default()
            });

            let provider = Provider::detect(&self.config.llm_endpoint);
            let (body, auth_header, auth_value) = match provider {
                Provider::OpenAI | Provider::Generic => {
                    let body = JsonValue::Object(vec![
                        ("model".to_string(), JsonValue::Str(model.to_string())),
                        (
                            "max_tokens".to_string(),
                            JsonValue::Number(max_tokens as f64),
                        ),
                        (
                            "messages".to_string(),
                            JsonValue::Array(vec![JsonValue::Object(vec![
                                ("role".to_string(), JsonValue::Str("user".to_string())),
                                ("content".to_string(), JsonValue::Str(prompt.to_string())),
                            ])]),
                        ),
                    ]);
                    (
                        body,
                        "Authorization",
                        format!("Bearer {}", self.config.api_key),
                    )
                }
                Provider::Anthropic => {
                    let body = JsonValue::Object(vec![
                        ("model".to_string(), JsonValue::Str(model.to_string())),
                        (
                            "max_tokens".to_string(),
                            JsonValue::Number(max_tokens as f64),
                        ),
                        (
                            "messages".to_string(),
                            JsonValue::Array(vec![JsonValue::Object(vec![
                                ("role".to_string(), JsonValue::Str("user".to_string())),
                                ("content".to_string(), JsonValue::Str(prompt.to_string())),
                            ])]),
                        ),
                    ]);
                    (body, "x-api-key", self.config.api_key.clone())
                }
                Provider::Gemini => {
                    let body = JsonValue::Object(vec![
                        (
                            "contents".to_string(),
                            JsonValue::Array(vec![JsonValue::Object(vec![
                                (
                                    "parts".to_string(),
                                    JsonValue::Array(vec![JsonValue::Object(vec![(
                                        "text".to_string(),
                                        JsonValue::Str(prompt.to_string()),
                                    )])]),
                                ),
                                ("role".to_string(), JsonValue::Str("user".to_string())),
                            ])]),
                        ),
                        (
                            "generationConfig".to_string(),
                            JsonValue::Object(vec![(
                                "maxOutputTokens".to_string(),
                                JsonValue::Number(max_tokens as f64),
                            )]),
                        ),
                    ]);
                    (body, "x-goog-api-key", self.config.api_key.clone())
                }
            };

            let mut req = agent
                .post(&self.config.llm_endpoint)
                .header("Content-Type", "application/json")
                .header(auth_header, &auth_value)
                .send_string(&stringify(&body))
                .map_err(|e| CidError::LlmError(format!("LLM request failed: {}", e)))?;

            let response_body = req
                .into_string()
                .map_err(|e| CidError::LlmError(format!("Failed to read LLM response: {}", e)))?;

            let json = parse_json(&response_body)
                .map_err(|e| CidError::ParseError(format!("Invalid LLM response JSON: {}", e)))?;

            Self::extract_response(&json, provider)
        }

        #[cfg(not(feature = "proxy"))]
        {
            let _ = (prompt, model, max_tokens);
            Err(CidError::LlmError(
                "Proxy not enabled. Build with: cargo build --features proxy".to_string(),
            ))
        }
    }

    #[cfg(feature = "proxy")]
    fn extract_response(json: &JsonValue, provider: Provider) -> CidResult<String> {
        match provider {
            Provider::OpenAI | Provider::Generic => {
                if let Some(choices) = json.get_array("choices") {
                    if let Some(first) = choices.first() {
                        if let Some(message) = first.get("message") {
                            if let Some(content) = message.get_str("content") {
                                return Ok(content.to_string());
                            }
                        }
                    }
                }
                Err(CidError::LlmError(
                    "Could not parse OpenAI response".to_string(),
                ))
            }
            Provider::Anthropic => {
                if let Some(content) = json.get_array("content") {
                    if let Some(first) = content.first() {
                        if let Some(text) = first.get_str("text") {
                            return Ok(text.to_string());
                        }
                    }
                }
                Err(CidError::LlmError(
                    "Could not parse Anthropic response".to_string(),
                ))
            }
            Provider::Gemini => {
                if let Some(candidates) = json.get_array("candidates") {
                    if let Some(first) = candidates.first() {
                        if let Some(content) = first.get("content") {
                            if let Some(parts) = content.get_array("parts") {
                                if let Some(first_part) = parts.first() {
                                    if let Some(text) = first_part.get_str("text") {
                                        return Ok(text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                Err(CidError::LlmError(
                    "Could not parse Gemini response".to_string(),
                ))
            }
        }
    }

    fn send_json(&self, stream: &mut TcpStream, status: u16, value: &JsonValue) -> CidResult<()> {
        let body = stringify(value);
        let status_text = match status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            status_text,
            body.len(),
            body
        );

        stream
            .write_all(response.as_bytes())
            .map_err(|e| CidError::IoError(format!("Failed to send response: {}", e)))?;

        Ok(())
    }

    fn send_error(stream: &mut TcpStream, status: u16, message: &str) -> CidResult<()> {
        let body = stringify(&JsonValue::Object(vec![(
            "error".to_string(),
            JsonValue::Str(message.to_string()),
        )]));

        let status_text = match status {
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            status_text,
            body.len(),
            body
        );

        stream
            .write_all(response.as_bytes())
            .map_err(|e| CidError::IoError(format!("Failed to send error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config() {
        let config = ProxyConfig::new(
            "127.0.0.1:8080",
            "https://api.openai.com/v1/chat/completions",
            "test-key",
        );
        assert_eq!(config.listen_addr, "127.0.0.1:8080");
        assert!(config.validate_responses);
    }

    #[test]
    fn test_json_parse() {
        let json = r#"{"text":"hello","context":"test"}"#;
        let value = parse_json(json).unwrap();
        assert_eq!(value.get_str("text"), Some("hello"));
        assert_eq!(value.get_str("context"), Some("test"));
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_provider_detect_openai() {
        assert_eq!(
            Provider::detect("https://api.openai.com/v1/chat/completions"),
            Provider::OpenAI
        );
        assert_eq!(
            Provider::detect("https://my-proxy.com/chat/completions"),
            Provider::OpenAI
        );
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_provider_detect_anthropic() {
        assert_eq!(
            Provider::detect("https://api.anthropic.com/v1/messages"),
            Provider::Anthropic
        );
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_provider_detect_gemini() {
        assert_eq!(
            Provider::detect("https://generativelanguage.googleapis.com/v1beta/models"),
            Provider::Gemini
        );
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_provider_detect_generic() {
        assert_eq!(
            Provider::detect("https://my-local-llm.com/api"),
            Provider::Generic
        );
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_extract_openai_response() {
        let json = parse_json(r#"{"choices":[{"message":{"content":"Hello!"}}]}"#).unwrap();
        assert_eq!(
            ProxyServer::extract_response(&json, Provider::OpenAI).unwrap(),
            "Hello!"
        );
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_extract_anthropic_response() {
        let json = parse_json(r#"{"content":[{"type":"text","text":"Hello!"}]}"#).unwrap();
        assert_eq!(
            ProxyServer::extract_response(&json, Provider::Anthropic).unwrap(),
            "Hello!"
        );
    }

    #[cfg(feature = "proxy")]
    #[test]
    fn test_extract_gemini_response() {
        let json =
            parse_json(r#"{"candidates":[{"content":{"parts":[{"text":"Hello!"}]}}]}"#).unwrap();
        assert_eq!(
            ProxyServer::extract_response(&json, Provider::Gemini).unwrap(),
            "Hello!"
        );
    }
}
