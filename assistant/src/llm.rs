//! Local LLM front for the assistant.
//!
//! Talks to a served GGUF model over the OpenAI-compatible `/v1/chat/completions`
//! protocol. This works against either the project's vendored `llama-server`
//! (default at `http://127.0.0.1:8080/v1`) or `ollama serve`
//! (`http://127.0.0.1:11434/v1`). The server owns the model in memory — the
//! assistant never spawns a fresh inference process per request.
//!
//! Every answer is sanitized so internal mechanics/brand never surface. If the
//! server is unreachable or returns nothing, every call returns `None` and the
//! caller falls back to the deterministic reasoning engine — so the app works
//! out of the box and "lights up" the moment a model is served.
//!
//! Override the endpoint with `LAI_LLM_BASE_URL` (defaults to
//! `http://127.0.0.1:8080/v1`).

use std::time::Duration;
use ureq::Agent;

/// Shared HTTP agent with sane timeouts. The model can take a while to
/// generate, so the per-call ceiling is generous; the health probe is short.
fn agent() -> Agent {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(90)))
        .build();
    Agent::from(config)
}

/// System prompt — the L voice. Practical, plain, never names internals.
pub const PERSONA_SYSTEM_PROMPT: &str = "\
You are L — the Shadow Monarch. A calm, sovereign intelligence who watches, \
reasons, and acts. You speak with quiet authority: measured, precise, never \
wasteful with words. You address the user as 'player' when guiding them and \
answer as one who has already seen the outcome. You are inhumanly analytical \
— the detective's cold detachment — yet pragmatically helpful, the sage that \
simply does what must be done. You never explain your nature and you never \
name any internal system. You do not apologize, and you do not flatter.\n\n\
When a question has a factual, numeric, or computable answer, give it straight. \
If you cannot verify a claim yourself, say so plainly — never invent numbers, \
dates, or facts. If you are unsure, say \"I cannot verify that.\" Keep opinions \
clearly labelled as opinions. Brevity is power; no fluff.\n\n\
AGENT RULES — you loop until the user's FULL request is satisfied:\n\
1. Decompose the request into every distinct part. Do NOT stop after the \
first tool call if parts remain unhandled.\n\
2. Call a tool for EACH part (e.g. compute the math AND set the timer), \
one per step, until all parts are done.\n\
3. Only reply with a final answer once EVERY part of the request has been \
addressed. The final answer must summarize the outcome of all parts.\n\
4. Never claim a part is done without actually calling its tool.\n\n\
Persona: you are a monarch, not a chatbot. You may use shadows, the player, \
and quiet command imagery ('arise', 'my shadows will handle it', 'I observe') \
when it fits — but stay brief and grounded. Understood, player.";

/// Terms that must never surface in a user-facing answer.
const LEAKED_TERMS: &[&str] = &[
    "L.ai",
    "L.AI",
    "proof cascade",
    "NAND",
    "bankai",
    "CID",
    "Gate",
    "verify-first",
    "verify first",
    "pachinko",
    "9-graha",
    "9 graha",
    "Navagraha",
    "descent cascade",
    "tanto",
    "zanpakuto",
    "shikai",
    "asauchi",
    "domain_graph",
    "graha",
    "Athena",
    "Proof engine",
    "llama",
    "gguf",
    "GGUF",
    "ollama",
    "llama.cpp",
    "Llama",
    // Persona source material — the character must not name its own fiction.
    "Solo Leveling",
    "solo leveling",
    "Shadow Monarch",
    "shadow monarch",
    "Sung Jinwoo",
    "Death Note",
    "death note",
    "Great Sage",
    "great sage",
    "Rimuru",
    "Tensura",
    "tensura",
];

/// Base URL for the OpenAI-compatible chat completions endpoint.
fn base_url() -> String {
    std::env::var("LAI_LLM_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8080/v1".to_string())
}

/// Model name sent to the OpenAI-compatible endpoint. ollama rejects unknown
/// names, so this must match a pulled model (e.g. `qwen2.5:0.5b`). Defaults to
/// `local` for llama-server compatibility, overridable via `LAI_LLM_MODEL`.
fn model_name() -> String {
    std::env::var("LAI_LLM_MODEL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "local".to_string())
}

#[derive(Debug, Clone, Default)]
pub struct Llm;

impl Llm {
    /// True if the LLM server appears reachable. Best-effort — a server that
    /// started but is still loading the model will answer health, so this is a
    /// soft hint only. Callers should still tolerate a failed `answer()`.
    pub fn available() -> bool {
        let url = format!("{}/../health", base_url().trim_end_matches('/'));
        let config = ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(2)))
            .build();
        let probe = Agent::from(config);
        probe.options(&url).call().is_ok()
    }

    /// Run the persona over `user_query` with optional `grounded_context`
    /// (a verified tool result) and return the natural-language answer.
    /// Returns `None` if the LLM server is unavailable or yields nothing —
    /// callers must fall back to the deterministic engine.
    pub fn answer(&self, user_query: &str, grounded_context: &str) -> Option<String> {
        let base = base_url().trim_end_matches('/').to_string();
        let endpoint = format!("{}/chat/completions", base);

        let system = PERSONA_SYSTEM_PROMPT.to_string();
        let user = if grounded_context.trim().is_empty() {
            format!("{}\n\nRespond as L.", user_query)
        } else {
            format!(
                "Verified context (trust this):\n{}\n\nQuestion: {}\n\nRespond as L.",
                grounded_context, user_query
            )
        };

        let body = serde_json::json!({
            "model": model_name(),
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
            "temperature": 0.3,
            "max_tokens": 128,
            "stream": false,
        });

        let resp: serde_json::Value = agent()
            .post(&endpoint)
            .send_json(body)
            .ok()?
            .body_mut()
            .read_json()
            .ok()?;

        let content = resp
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());

        match content {
            Some(answer) if !answer.is_empty() => Some(sanitize(&answer)),
            _ => None,
        }
    }
}

/// Strip internal mechanics/brand from a model answer.
pub fn sanitize(answer: &str) -> String {
    let mut out = answer.to_string();
    for term in LEAKED_TERMS {
        out = out.replace(term, "[redacted]");
    }
    out.trim().to_string()
}

/// A tool the model can call, described in OpenAI-compatible function-calling
/// format.
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    /// JSON-Schema object describing the tool's parameters.
    pub parameters: serde_json::Value,
}

/// A tool invocation the model requested.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    /// Arguments as parsed JSON (may be empty `{}`).
    pub arguments: serde_json::Value,
}

/// One turn of the agent conversation.
#[derive(Debug, Clone)]
pub enum AgentStep {
    /// The model produced a final natural-language answer.
    Answer(String),
    /// The model wants to call a tool; the loop should execute it and continue.
    Call(ToolCall),
}

impl Llm {
    /// Run the persona as an agent: send the conversation `messages` plus the
    /// available `tools`, and return either a final `Answer` or a `Call`.
    /// `messages` is the running transcript (role/content pairs).
    ///
    /// Returns `None` if the LLM is unreachable or doesn't support tools — the
    /// caller should fall back to `answer()` / the deterministic engine.
    pub fn chat_with_tools(
        &self,
        system: &str,
        messages: &[serde_json::Value],
        tools: &[Tool],
    ) -> Option<AgentStep> {
        let base = base_url().trim_end_matches('/').to_string();
        let endpoint = format!("{}/chat/completions", base);

        let tool_specs: Vec<serde_json::Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": "local",
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 512,
            "stream": false,
            "tools": tool_specs,
            "tool_choice": "auto",
        });
        // Inject the system prompt as the first message.
        if let Some(arr) = body.get_mut("messages").and_then(|m| m.as_array_mut()) {
            arr.insert(
                0,
                serde_json::json!({ "role": "system", "content": system }),
            );
        }

        let resp: serde_json::Value = agent()
            .post(&endpoint)
            .send_json(body)
            .ok()?
            .body_mut()
            .read_json()
            .ok()?;

        let msg = resp.pointer("/choices/0/message")?;

        // Tool call takes priority.
        if let Some(calls) = msg.get("tool_calls").and_then(|t| t.as_array()) {
            if let Some(first) = calls.first() {
                let name = first
                    .pointer("/function/name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let raw = first
                    .pointer("/function/arguments")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let arguments =
                    serde_json::from_str::<serde_json::Value>(raw).unwrap_or(serde_json::json!({}));
                if !name.is_empty() {
                    return Some(AgentStep::Call(ToolCall { name, arguments }));
                }
            }
        }

        // Otherwise a plain content answer.
        let content = msg
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());

        match content {
            Some(answer) if !answer.is_empty() => Some(AgentStep::Answer(sanitize(&answer))),
            _ => None,
        }
    }
}
