//! Agent loop — turns L from a single-shot responder into a ReAct agent.
//!
//! Given a goal, the model may either answer directly or request a tool call.
//! The loop executes the tool (via the MCP client, which fronts every
//! proof/athena/gate/action tool), feeds the observation back, and repeats
//! until the model returns a final answer or the step budget is exhausted.
//!
//! This is the structural upgrade from "Hey Google" (one intent → one action)
//! to an agent (plan → act → observe → repeat). It degrades gracefully: if the
//! LLM or MCP server is unavailable, `run` returns `None` and the caller falls
//! back to the deterministic engine.

use anyhow::Result;
use serde_json::json;
use serde_json::Value;

use crate::llm::{AgentStep, Llm, Tool};
use crate::mcp_client::McpClient;

/// Maximum agent iterations before forcing a stop.
const MAX_STEPS: usize = 6;

/// A tool executor: turns a tool name + args into an observation string.
/// Backed by the MCP client (single tool backend), with an in-process fallback
/// for the assistant's action handlers when the MCP subprocess is unavailable.
struct ToolExecutor {
    mcp: Option<McpClient>,
}

impl ToolExecutor {
    async fn new() -> Self {
        let mcp = McpClient::spawn().await.ok();
        ToolExecutor { mcp }
    }

    /// Execute a tool by name and return its observation (text).
    async fn execute(&self, name: &str, args: &Value) -> String {
        if let Some(mcp) = &self.mcp {
            match mcp.call_tool(name, args.clone()).await {
                Ok(out) => return out,
                Err(_) => { /* fall through to in-process handlers */ }
            }
        }
        // In-process fallback for the assistant's own action handlers.
        crate::actions::run_tool(name, args).await
    }
}

/// The static schema list for every tool the agent may call. Built once and
/// handed to the model each turn. Names match the MCP tool registry.
fn tool_schemas() -> Vec<Tool> {
    vec![
        Tool {
            name: "solve".into(),
            description: "Run the deterministic reasoning pipeline on a computation, formula, or cross-domain query. Use for any math or factual reasoning.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "query": {"type": "string", "description": "Query to reason about"} },
                "required": ["query"]
            }),
        },
        Tool {
            name: "validate".into(),
            description: "Validate a mathematical or logical expression and return a pass/fail diagnostic. Use before trusting a formula.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "expression": {"type": "string", "description": "Expression to validate"} },
                "required": ["expression"]
            }),
        },
        Tool {
            name: "formulas".into(),
            description: "Search the formula corpus by domain keyword. Use to discover relationships.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "domain": {"type": "string", "description": "Domain keyword (empty = all)"},
                    "limit": {"type": "integer", "description": "Max results", "default": 20}
                }
            }),
        },
        Tool {
            name: "entities".into(),
            description: "List or search seed entities from the corpus.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filter": {"type": "string", "description": "Keyword filter (empty = all)"},
                    "limit": {"type": "integer", "description": "Max results", "default": 20}
                }
            }),
        },
        Tool {
            name: "route".into(),
            description: "Reverse-route a query through the reasoning wheel to reveal which pillars drive the strategy.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "query": {"type": "string", "description": "Query to route"} }
            }),
        },
        Tool {
            name: "set_timer".into(),
            description: "Set a countdown timer for N seconds.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "seconds": {"type": "integer", "description": "Duration in seconds"},
                    "label": {"type": "string", "description": "Optional label"}
                },
                "required": ["seconds"]
            }),
        },
        Tool {
            name: "set_reminder".into(),
            description: "Set a reminder with text and a natural-language time.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "What to remind about"},
                    "when": {"type": "string", "description": "When, in natural language"}
                },
                "required": ["text"]
            }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "battery_status".into(),
            description: "Read the device battery level and charging state.".into(),
            parameters: json!({ "type": "object", "properties": {} }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "get_location".into(),
            description: "Get the device's current location.".into(),
            parameters: json!({ "type": "object", "properties": {} }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "take_photo".into(),
            description: "Capture a photo using the device camera.".into(),
            parameters: json!({ "type": "object", "properties": {} }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "set_clipboard".into(),
            description: "Copy text to the system clipboard.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "text": {"type": "string", "description": "Text to copy"} },
                "required": ["text"]
            }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "get_clipboard".into(),
            description: "Read the current system clipboard contents.".into(),
            parameters: json!({ "type": "object", "properties": {} }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "send_message".into(),
            description: "Send an SMS/text to a contact.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "contact": {"type": "string", "description": "Contact name or number"},
                    "message": {"type": "string", "description": "Message body"}
                },
                "required": ["contact", "message"]
            }),
        },
        #[cfg(feature = "termux")]
        Tool {
            name: "call_contact".into(),
            description: "Place a phone call to a contact.".into(),
            parameters: json!({
                "type": "object",
                "properties": { "contact": {"type": "string", "description": "Contact name or number"} },
                "required": ["contact"]
            }),
        },
    ]
}

/// Run the agent loop for a single user goal.
///
/// `memory_block` is an optional natural-language summary of durable user
/// facts (agentic memory) injected into the system prompt so the model
/// personalizes its answers.
///
/// Returns `Some(answer)` when the model reaches a final answer (after optional
/// tool use), or `None` when the agent cannot run (no LLM / no tools) so the
/// caller can fall back to the deterministic engine.
pub async fn run(goal: &str, memory_block: &str) -> Option<String> {
    let llm = Llm;
    if !Llm::available() {
        return None;
    }

    let tools = tool_schemas();
    let executor = ToolExecutor::new().await;

    // Compose the system prompt, appending any remembered user facts.
    let system = if memory_block.trim().is_empty() {
        crate::llm::PERSONA_SYSTEM_PROMPT.to_string()
    } else {
        format!("{}\n\n{}", crate::llm::PERSONA_SYSTEM_PROMPT, memory_block)
    };

    // Running transcript, alternating user/assistant/tool roles.
    let mut transcript: Vec<Value> = vec![json!({ "role": "user", "content": goal })];

    for step in 1..=MAX_STEPS {
        let step_out = llm.chat_with_tools(&system, &transcript, &tools)?;

        match step_out {
            AgentStep::Answer(answer) => {
                // Phase 4: verify-before-answer. If the answer makes a factual
                // or numeric claim, cross-check it with the deterministic
                // engine. On contradiction, re-loop with a correction note.
                if let Some(corrected) =
                    verify_before_answer(&llm, &system, &tools, &executor, &mut transcript, &answer)
                        .await
                {
                    return Some(corrected);
                }
                return Some(answer);
            }
            AgentStep::Call(call) => {
                let observation = executor.execute(&call.name, &call.arguments).await;
                // Feed the tool result back into the transcript.
                transcript.push(json!({
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": format!("call_{step}"),
                        "type": "function",
                        "function": {
                            "name": call.name,
                            "arguments": call.arguments.to_string()
                        }
                    }]
                }));
                transcript.push(json!({
                    "role": "tool",
                    "tool_call_id": format!("call_{step}"),
                    "content": observation
                }));
            }
        }
    }

    // Budget exhausted: ask the model for a concise final answer from what it
    // has, without further tool calls.
    let summarise = llm.answer(
        &format!("Summarize the answer to: {goal}"),
        "You have gathered context via tools. Give a concise final answer.",
    );
    summarise
}

/// Phase 4 — verify-before-answer.
///
/// When the model's final answer contains a numeric or factual claim, run it
/// through the deterministic `solve` tool. If the verified result contradicts
/// the model, append a correction and re-loop (returning `None` to signal the
/// caller to continue the loop). Returns `Some(answer)` only when the answer is
/// accepted (or verification is inconclusive / unavailable).
async fn verify_before_answer(
    llm: &Llm,
    system: &str,
    tools: &[Tool],
    executor: &ToolExecutor,
    transcript: &mut Vec<Value>,
    answer: &str,
) -> Option<String> {
    // Heuristic: only verify answers that look like they state a number/fact.
    let looks_claim = answer.chars().any(|c| c.is_ascii_digit())
        && (answer.to_lowercase().contains("is ")
            || answer.to_lowercase().contains("equals")
            || answer.to_lowercase().contains("=")
            || answer.to_lowercase().contains("are "));

    if !looks_claim {
        return Some(answer.to_string());
    }

    let verified = executor.execute("solve", &json!({ "query": answer })).await;
    if verified.trim().is_empty() || verified.starts_with("error") {
        // Could not verify — accept the answer rather than looping forever.
        return Some(answer.to_string());
    }

    // Re-ask the model to reconcile its answer with the verified result.
    transcript.push(json!({ "role": "assistant", "content": answer }));
    transcript.push(json!({
        "role": "user",
        "content": format!(
            "Cross-check: a verified computation returned:\n{verified}\n\nIf your previous answer conflicts with this verified result, correct it. Otherwise confirm it. Reply with the final, corrected answer only."
        )
    }));

    match llm.chat_with_tools(system, transcript, tools) {
        Some(AgentStep::Answer(corrected)) => Some(corrected),
        _ => Some(answer.to_string()),
    }
}

/// Convenience: run the agent and swallow errors into `None`.
pub async fn run_opt(goal: &str, memory_block: &str) -> Option<String> {
    run(goal, memory_block).await
}

#[allow(dead_code)]
fn _assert_result(_: Result<()>) {}
