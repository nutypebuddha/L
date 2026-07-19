//! MCP client for the assistant.
//!
//! Spawns the project's `lai mcp` server (stdio JSON-RPC) as a child process
//! and speaks the Model Context Protocol over its stdin/stdout. This is the
//! single tool backend for the agent loop: every proof/athena/gate tool
//! (`solve`, `validate`, `formulas`, `route`, `optimize`, `build`, `chart`,
//! `entity_get`, `entities`, plus the assistant action tools) is reachable as
//! a callable function.
//!
//! If the `lai mcp` subprocess cannot be started (e.g. a static binary with no
//! sibling `lai` executable, as on Android), every call returns `Err` and the
//! agent loop degrades to its in-process action handlers.

use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;

/// Locate the `lai` executable that serves `lai mcp`.
/// Prefers `LAI_BIN` (set by the daemon), then the current exe's directory,
/// then a `lai` on PATH.
fn lai_bin() -> String {
    if let Ok(env) = std::env::var("LAI_BIN") {
        if !env.is_empty() {
            return env;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("lai");
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    "lai".to_string()
}

/// A live MCP session over a spawned `lai mcp` process.
pub struct McpClient {
    child: Mutex<Option<Child>>,
    stdin: Mutex<ChildStdin>,
    reader: Mutex<Box<dyn tokio::io::AsyncBufRead + Send + Unpin>>,
    next_id: Mutex<u64>,
}

impl McpClient {
    /// Spawn `lai mcp` and complete the MCP handshake.
    pub async fn spawn() -> Result<Self> {
        let bin = lai_bin();
        let mut child = Command::new(&bin)
            .arg("mcp")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("failed to spawn `{bin} mcp`: {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("mcp child has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("mcp child has no stdout"))?;

        let reader: Box<dyn tokio::io::AsyncBufRead + Send + Unpin> =
            Box::new(BufReader::new(stdout));

        let client = Self {
            child: Mutex::new(Some(child)),
            stdin: Mutex::new(stdin),
            reader: Mutex::new(reader),
            next_id: Mutex::new(1),
        };

        // Handshake: initialize + notifications/initialized.
        client
            .request(
                "initialize",
                serde_json::json!({
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {"name": "lai-assistant", "version": "0.1.0"}
                }),
            )
            .await?;
        client
            .notify("notifications/initialized", serde_json::json!({}))
            .await?;

        Ok(client)
    }

    /// List available tool names + schemas.
    pub async fn list_tools(&self) -> Result<Vec<Value>> {
        let resp = self.request("tools/list", serde_json::json!({})).await?;
        let tools = resp
            .pointer("/result/tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(tools)
    }

    /// Call a tool by name with JSON arguments; returns the tool's text output.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String> {
        let resp = self
            .request(
                "tools/call",
                serde_json::json!({ "name": name, "arguments": arguments }),
            )
            .await?;

        if let Some(err) = resp.get("error") {
            return Err(anyhow!("mcp tool error: {err}"));
        }
        let text = resp
            .pointer("/result/content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        Ok(text)
    }

    // ── JSON-RPC plumbing ──────────────────────────────────────

    async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let id = {
            let mut n = self.next_id.lock().await;
            *n += 1;
            *n
        };
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_line(&req.to_string()).await?;

        // Read until we get a response with our id.
        let mut reader = self.reader.lock().await;
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("mcp server closed connection"));
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(msg) = serde_json::from_str::<Value>(trimmed) {
                if msg.get("id").and_then(|v| v.as_u64()) == Some(id) {
                    return Ok(msg);
                }
                // server might emit notifications; ignore and keep reading
            }
        }
    }

    async fn notify(&self, method: &str, params: Value) -> Result<()> {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_line(&req.to_string()).await
    }

    async fn write_line(&self, line: &str) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Best-effort kill; the lock is not held across drop in practice.
        if let Some(child) = self.child.get_mut().take() {
            let mut cmd = child;
            let _ = cmd.start_kill();
        }
    }
}
