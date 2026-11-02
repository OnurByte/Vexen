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
            let mut clipped: String = message.chars().take(500).collect();
            while serde_json::to_string(&clipped).unwrap().len() > 1700 {
                clipped.pop();
            }
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
            for key in [
                "id", "event_id", "status", "sequence", "kind", "type", "text",
            ] {
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
    // Some clients only expose text content to the model. Keep the actual
    // qualified lesson there as well as in structuredContent. When necessary,
    // shorten the longest lesson without discarding record identity or status.
    let mut brief = serde_json::Value::Object(brief);
    loop {
        let text =
            serde_json::to_string(&brief).unwrap_or_else(|_| "{\"status\":\"error\"}".into());
        if text.len() <= 1700 {
            return text;
        }
        let items = brief["items"].as_array_mut().expect("items is an array");
        let longest = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                item["text"]
                    .as_str()
                    .filter(|text| !text.is_empty())
                    .map(|text| (i, text.len()))
            })
            .max_by_key(|(_, len)| *len)
            .map(|(i, _)| i);
        let Some(i) = longest else {
            return "{\"status\":\"response_too_large\"}".into();
        };
        let mut lesson = items[i]["text"].as_str().unwrap().to_owned();
        lesson.pop(); // Remove a complete Unicode scalar, never half a UTF-8 character.
        items[i]["text"] = serde_json::Value::String(lesson);
        items[i]["text_truncated"] = serde_json::Value::Bool(true);
    }
}

/// Serve the MCP stdio protocol. Protocol messages are written only by rmcp;
/// this function emits no diagnostics on stdout.
pub async fn serve_stdio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = VexenServer::new().serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_error_messages_fit_the_utf8_text_budget() {
        let result = response(Err(super::super::McpInputError::Invalid("🦀".repeat(1000))));
        let value = serde_json::to_value(result).unwrap();
        let text = value["content"][0]["text"].as_str().unwrap();
        assert!(text.len() <= 1700, "error is {} bytes", text.len());
        assert!(text.starts_with("invalid request:"));
    }

    #[test]
    fn text_only_clients_receive_the_qualified_failure_lesson() {
        let value = serde_json::json!({
            "schema_version": 1, "repo_id": "repo", "status": "observed_failure",
            "items": [{"id": "event", "status": "observed_failure",
                "text": "Observed failure (applicability checked; not a guarantee) for pricing.py: \"Use ROUND_HALF_UP; 0.125 must become 0.13.\""}]
        });
        let brief: serde_json::Value = serde_json::from_str(&compact_text(&value)).unwrap();
        assert_eq!(brief["items"][0]["text"], value["items"][0]["text"]);
        assert_eq!(brief["items"][0]["id"], "event");
    }

    #[test]
    fn long_unicode_lessons_keep_status_and_identity_within_text_budget() {
        let value = serde_json::json!({
            "schema_version": 1, "repo_id": "r".repeat(64), "status": "observed_failure",
            "items": (0..3).map(|i| serde_json::json!({
                "id": format!("{i:064}"), "status": "conflicted",
                "text": format!("Conflicting observed outcomes; abstain: {}", "\\\"ü🦀".repeat(100))
            })).collect::<Vec<_>>()
        });
        let text = compact_text(&value);
        assert!(text.len() <= 1700);
        let brief: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(brief["status"], "observed_failure");
        assert_eq!(brief["items"].as_array().unwrap().len(), 3);
        for (i, item) in brief["items"].as_array().unwrap().iter().enumerate() {
            assert_eq!(item["id"], format!("{i:064}"));
            assert!(
                item["text"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("Conflicting observed outcomes; abstain:")
            );
            assert_eq!(item["text_truncated"], true);
        }
    }
}
