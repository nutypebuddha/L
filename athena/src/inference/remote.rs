//! # RemoteBackend — HTTP API inference (ollama / llama.cpp / OpenAI-compatible)
//!
//! Connects to any OpenAI-compatible inference server:
//! - llama.cpp server (`http://127.0.0.1:8080/v1`)
//! - Ollama (`http://127.0.0.1:11434/v1`)
//! - OpenAI API (with api_key)
//!
//! Uses `ureq` for blocking HTTP — no async runtime needed at CLI level.

use crate::inference::config::{InferenceConfig, DEFAULT_ENDPOINT_URL};
use crate::inference::{
    BackendKind, Capability, HealthStatus, InferenceBackend, InferenceError, InferenceRequest,
    InferenceResponse,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Remote backend connecting to an OpenAI-compatible inference API.
pub struct RemoteBackend {
    config: InferenceConfig,
    endpoint: String,
    client: ureq::Agent,
}

impl RemoteBackend {
    pub fn new(config: InferenceConfig) -> Self {
        let endpoint = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| DEFAULT_ENDPOINT_URL.to_string());

        let client = ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .timeout_global(Some(Duration::from_secs(60)))
                .build(),
        );

        Self {
            config,
            endpoint,
            client,
        }
    }

    /// POST to the chat completions endpoint.
    fn chat_completion(
        &self,
        request: InferenceRequest,
    ) -> Result<InferenceResponse, InferenceError> {
        let url = format!("{}/chat/completions", self.endpoint.trim_end_matches('/'));

        let system = request
            .system_prompt
            .as_deref()
            .unwrap_or(&self.config.system_prompt);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            },
        ];

        let body = ChatCompletionRequest {
            model: "default".to_string(),
            messages,
            temperature: request.temperature.unwrap_or(self.config.temperature),
            max_tokens: request.max_tokens.unwrap_or(self.config.max_tokens) as u32,
            top_p: self.config.top_p,
            stop: if self.config.stop.is_empty() {
                None
            } else {
                Some(self.config.stop.clone())
            },
            stream: false,
        };

        let body_json = serde_json::to_string(&body)
            .map_err(|e| InferenceError::InferenceFailed(format!("serialization error: {}", e)))?;

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add API key if configured
        if let Some(ref key) = self.config.api_key {
            req = req.header("Authorization", &format!("Bearer {}", key));
        }

        let response: ChatCompletionResponse = req
            .send(&body_json)
            .map_err(|e| {
                InferenceError::BackendUnavailable(format!("HTTP request to {} failed: {}", url, e))
            })?
            .into_body()
            .read_to_string()
            .map_err(|e| InferenceError::InferenceFailed(format!("response read failed: {}", e)))
            .and_then(|body| {
                serde_json::from_str(&body).map_err(|e| {
                    InferenceError::InferenceFailed(format!("response parsing failed: {}", e))
                })
            })?;

        let text = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let tokens_generated = response.usage.map(|u| u.completion_tokens).unwrap_or(0) as usize;
        let finish_reason = response
            .choices
            .first()
            .map(|c| c.finish_reason.clone())
            .unwrap_or_default();

        Ok(InferenceResponse {
            text: text.trim().to_string(),
            tokens_generated,
            tokens_per_second: 0.0, // would need timing from server
            finish_reason,
            raw: None,
        })
    }

    /// Health check against the server.
    fn health_check(&self) -> Result<HealthStatus, InferenceError> {
        let base = self.endpoint.trim_end_matches('/');
        let base = base.strip_suffix("/v1").unwrap_or(base);

        // Try /health endpoint (llama.cpp style)
        let health_url = format!("{}/health", base);
        match self.client.get(&health_url).call() {
            Ok(resp) => {
                if let Ok(body) = resp.into_body().read_to_string() {
                    if let Ok(health) = serde_json::from_str::<HealthResponse>(&body) {
                        return Ok(HealthStatus {
                            healthy: true,
                            model_loaded: health.model_path,
                            backend_kind: BackendKind::Remote,
                            context_size: None,
                            message: format!(
                                "Remote backend healthy: {} slots available",
                                health.slots_idle.unwrap_or(0)
                            ),
                        });
                    }
                }
                Ok(HealthStatus {
                    healthy: true,
                    model_loaded: None,
                    backend_kind: BackendKind::Remote,
                    context_size: None,
                    message: "Remote backend responding (basic)".to_string(),
                })
            }
            Err(e) => {
                // Fallback: try a simple /v1/models GET
                let models_url = format!("{}/models", base);
                match self.client.get(&models_url).call() {
                    Ok(_) => Ok(HealthStatus {
                        healthy: true,
                        model_loaded: None,
                        backend_kind: BackendKind::Remote,
                        context_size: None,
                        message: "Remote backend responding (OpenAI API)".to_string(),
                    }),
                    Err(_) => Ok(HealthStatus {
                        healthy: false,
                        model_loaded: None,
                        backend_kind: BackendKind::Remote,
                        context_size: None,
                        message: format!("Health check failed: {}", e),
                    }),
                }
            }
        }
    }
}

impl InferenceBackend for RemoteBackend {
    fn generate(&mut self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        self.chat_completion(request)
    }

    fn health(&self) -> HealthStatus {
        self.health_check().unwrap_or_else(|e| HealthStatus {
            healthy: false,
            model_loaded: None,
            backend_kind: BackendKind::Remote,
            context_size: None,
            message: format!("Health check failed: {}", e),
        })
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "openai-compatible".to_string(),
            version: None,
            supported_features: vec![
                "remote".to_string(),
                "chat-completions".to_string(),
                "streaming".to_string(),
            ],
        }]
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Remote
    }
}

// ─── API types ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
    top_p: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    finish_reason: String,
}

/// Mirrors the OpenAI-compatible `usage` object; unread fields kept so the
/// wire shape stays documented.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<ChatUsage>,
}

/// Mirrors llama.cpp's `/health` response; unread fields kept for wire shape.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HealthResponse {
    #[serde(default)]
    model_path: Option<String>,
    #[serde(default)]
    slots_idle: Option<u32>,
    #[serde(default)]
    slots_processing: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::config::InferenceConfig;

    #[test]
    fn test_remote_backend_capabilities() {
        let config = InferenceConfig::default();
        let backend = RemoteBackend::new(config);
        let caps = backend.capabilities();
        assert!(!caps.is_empty());
        assert_eq!(caps[0].name, "openai-compatible");
    }

    #[test]
    fn test_remote_backend_health_no_server() {
        // This will fail since there's no server running — that's expected
        let mut config = InferenceConfig::default();
        config.endpoint_url = Some("http://127.0.0.1:9999/v1".to_string());
        let backend = RemoteBackend::new(config);
        let health = backend.health();
        assert!(!health.healthy); // no server = not healthy
        assert_eq!(health.backend_kind, BackendKind::Remote);
    }
}
