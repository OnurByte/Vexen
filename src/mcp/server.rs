use super::{
    schema::{CheckInput, ExplainInput, OutcomeInput, RecordInput},
    tools,
};
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool_handler, tool_router,
};

#[derive(Clone)]
pub struct VexenServer {
    tool_router: ToolRouter<Self>,
}

impl Default for VexenServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for VexenServer {}

#[tool_router(router = tool_router)]
impl VexenServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[rmcp::tool(name="vexen_check", description="Check applicable, evidence-backed repository memory. Does not change repository state.", annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false), output_schema = rmcp::handler::server::tool::schema_for_type::<super::Envelope<serde_json::Value>>())]
    pub async fn check(&self, Parameters(input): Parameters<CheckInput>) -> CallToolResult {
        response(tools::check(input))
    }

    #[rmcp::tool(name="vexen_explain", description="Explain the provenance chain for a record without executing its content.", annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false), output_schema = rmcp::handler::server::tool::schema_for_type::<super::Envelope<serde_json::Value>>())]
    pub async fn explain(&self, Parameters(input): Parameters<ExplainInput>) -> CallToolResult {
        response(tools::explain(input))
    }

    #[rmcp::tool(name="vexen_outcome", description="Append allowlisted observed test metadata or supersede an older record.", annotations(read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false), output_schema = rmcp::handler::server::tool::schema_for_type::<super::Envelope<serde_json::Value>>())]
    pub async fn outcome(&self, Parameters(input): Parameters<OutcomeInput>) -> CallToolResult {
        response(tools::outcome(input))
    }

    #[rmcp::tool(name="vexen_record", description="Append a candidate or human accepted decision to this checkout's evidence store.", annotations(read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false), output_schema = rmcp::handler::server::tool::schema_for_type::<super::Envelope<serde_json::Value>>())]
    pub async fn record(&self, Parameters(input): Parameters<RecordInput>) -> CallToolResult {
        response(tools::record(input))
    }
}

fn response(
    result: std::result::Result<super::Envelope<serde_json::Value>, super::McpInputError>,
) -> CallToolResult {
    match result {
        Ok(value) => {
            let structured = serde_json::to_value(&value).unwrap_or(serde_json::Value::Null);
            let text = compact_text(&structured);
            let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
            result.structured_content = Some(structured);
            result
        }
        Err(error) => {
            let message = error.to_string();
            let clipped: String = message.chars().take(500).collect();
            CallToolResult::error(vec![ContentBlock::text(clipped)])
        }
    }
}

fn compact_text(value: &serde_json::Value) -> String {
    let mut brief = serde_json::Map::new();
    for key in ["schema_version", "repo_id", "status", "reason"] {
        if let Some(v) = value.get(key) {
            brief.insert(key.to_owned(), v.clone());
        }
    }
    let items = value
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let brief_items: Vec<_> = items
        .iter()
        .take(12)
        .map(|item| {
            let mut entry = serde_json::Map::new();
            for key in ["id", "event_id", "status", "sequence", "kind", "type"] {
                if let Some(v) = item.get(key) {
                    entry.insert(key.to_owned(), v.clone());
                }
            }
            serde_json::Value::Object(entry)
        })
        .collect();
    brief.insert("items".into(), serde_json::Value::Array(brief_items));
    if items.len() > 12 {
        brief.insert(
            "truncated_items".into(),
            serde_json::Value::from(items.len() - 12),
        );
    }
    let text = serde_json::to_string(&serde_json::Value::Object(brief))
        .unwrap_or_else(|_| "{\"status\":\"error\"}".into());
    if text.len() <= 1700 {
        text
    } else {
        "{\"status\":\"response_too_large\"}".into()
    }
}

/// Serve the MCP stdio protocol. Protocol messages are written only by rmcp;
/// this function emits no diagnostics on stdout.
pub async fn serve_stdio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = VexenServer::new().serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
