use std::borrow::Cow;
use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::ErrorData;
use rmcp::RoleServer;

use super::compact::{compact_data, estimate_tokens, Detail, DEFAULT_LIMIT};
use super::{AthenaMCP, AthenaRequest};

pub struct McpHandler {
    inner: Arc<AthenaMCP>,
}

impl McpHandler {
    pub fn new(mcp: AthenaMCP) -> Self {
        McpHandler {
            inner: Arc::new(mcp),
        }
    }

    pub fn call_tool_sync(
        &self,
        name: &str,
        arguments: Option<&serde_json::Map<String, serde_json::Value>>,
    ) -> CallToolResult {
        let inner_method = name.strip_prefix("athena_").unwrap_or(name);
        let detail = Detail::from_params(arguments);
        let limit = arguments
            .and_then(|a| a.get("limit"))
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_LIMIT);
        let params = arguments
            .cloned()
            .map(serde_json::Value::Object)
            .unwrap_or(serde_json::Value::Null);

        let req = AthenaRequest {
            method: inner_method.to_string(),
            params,
        };
        let resp = self.inner.handle_request(req);

        if resp.success {
            // Baseline: what the pre-compaction server emitted (pretty-printed
            // full response). The delta to what we actually emit is the
            // context-token saving for the calling model.
            let baseline = serde_json::to_string_pretty(&resp.data).unwrap_or_default();
            let data = match detail {
                Detail::Full => resp.data,
                Detail::Compact => compact_data(inner_method, resp.data, limit),
            };
            let emitted = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());
            self.inner
                .savings
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .record(estimate_tokens(&baseline), estimate_tokens(&emitted));
            CallToolResult::success(vec![Content::text(emitted)])
        } else {
            let msg = resp.error.unwrap_or_else(|| "Unknown error".to_string());
            CallToolResult::error(vec![Content::text(msg)])
        }
    }

    fn tools_to_mcp(&self) -> Vec<Tool> {
        self.inner
            .tools
            .all()
            .iter()
            .map(|t| {
                let schema = match t.parameters() {
                    serde_json::Value::Object(map) => Arc::new(map.clone()),
                    _ => Arc::new(serde_json::Map::new()),
                };
                Tool::new(
                    Cow::Owned(t.name().to_string()),
                    Cow::Owned(t.description().to_string()),
                    schema,
                )
            })
            .collect()
    }
}

impl Clone for McpHandler {
    fn clone(&self) -> Self {
        McpHandler {
            inner: self.inner.clone(),
        }
    }
}

impl ServerHandler for McpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::new(
            "athena",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "Athena — relational intelligence engine. Formulas, not facts.\n\
             Use tools to validate, traverse, compose, reason, and evaluate across formula and entity graphs.\n\
             Responses are compacted to save your context tokens: lists cap at `limit` (default 25, an `omitted` \
             count marks truncation) and flavor text is trimmed. Pass detail:\"full\" for the complete response. \
             The `athena_savings` tool reports tokens saved this session.",
        )
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools_to_mcp()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(self.call_tool_sync(&request.name, request.arguments.as_ref()))
    }
}
