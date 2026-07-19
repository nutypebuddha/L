//! Assistant MCP server over stdio (JSON-RPC 2.0, line-delimited).
//!
//! Run with `lai assistant --mcp`. The process reads one JSON-RPC request per
//! line from stdin and writes one JSON-RPC response per line to stdout. This is
//! the transport the Android app uses to talk to the engine: no localhost
//! socket, no port, no auth surface — the app owns the child process's pipes.
//!
//! Exposes a single `chat` tool whose `arguments: { "text": "..." }` maps onto
//! `Assistant::process_text_json`, returning the same `AssistantResponse` the
//! HTTP `/chat` path returns.

use crate::Assistant;
use serde_json::Value;
use std::io::{BufRead, Write};

/// Start the stdio MCP loop. Returns when stdin hits EOF (parent closed the
/// pipe) or an unrecoverable read error occurs.
///
/// Must be called from within a Tokio runtime (`tokio::runtime::Handle::current`)
/// — the `Assistant` command handler runs us via `rt.block_on`, so we reuse that
/// runtime's handle rather than spawning a nested one (which panics).
pub fn serve_stdio(assist: std::sync::Arc<Assistant>) {
    eprintln!("[assistant] MCP stdio server starting on stdin/stdout");
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut buffer = String::new();
    let handle = tokio::runtime::Handle::current();

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => break, // EOF — parent closed stdin
            Ok(_) => {
                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some(resp) = tokio::task::block_in_place(|| handle.block_on(handle_jsonrpc(line, &assist))) {
                    let resp_str = serde_json::to_string(&resp).unwrap_or_default();
                    let _ = writeln!(writer, "{resp_str}");
                    let _ = writer.flush();
                }
            }
            Err(e) => {
                eprintln!("[assistant] MCP stdio read error: {e}");
                break;
            }
        }
    }
}

async fn handle_jsonrpc(line: &str, assist: &std::sync::Arc<Assistant>) -> Option<Value> {
    let msg: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => {
            return Some(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": "Parse error"},
                "id": null
            }));
        }
    };

    let method = msg["method"].as_str().unwrap_or("");
    let id = msg["id"].clone();
    let params = &msg["params"];

    match method {
        "initialize" => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "protocolVersion": "2025-11-25",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "lai-assistant", "version": env!("CARGO_PKG_VERSION") }
            },
            "id": id
        })),
        "notifications/initialized" => None,
        "tools/list" => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "tools": [
                    {
                        "name": "chat",
                        "title": "L.ai Assistant chat",
                        "description": "Send a message to L (the Shadow Monarch) and get a response. Routes intent, runs tools, and falls back to the local LLM or the deterministic reasoning engine.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string", "description": "The user's message" }
                            },
                            "required": ["text"]
                        }
                    }
                ]
            },
            "id": id
        })),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or("");
            if name != "chat" {
                return Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("unknown tool: {name}") }
                }));
            }
            let text = params["arguments"]["text"].as_str().unwrap_or("").to_string();
            let result = assist.process_text_json(&text).await;
            match result {
                Ok(resp) => Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [ { "type": "text", "text": resp.to_json() } ],
                        "isError": false
                    }
                })),
                Err(e) => Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": e.to_string() }
                })),
            }
        }
        _ => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("method not found: {method}") }
        })),
    }
}
