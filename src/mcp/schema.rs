use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInput {
    pub repo_root: String,
    pub paths: Option<Vec<String>>,
    pub diff_ref: Option<String>,
    pub query: Option<String>,
    pub conditions: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub repo_root: String,
    pub record: crate::store::NewEvent,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SupersedeInput {
    pub idempotency_key: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeInput {
    pub repo_root: String,
    pub event_id: String,
    pub outcome: Option<crate::store::NewOutcome>,
    pub supersede: Option<SupersedeInput>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainInput {
    pub repo_root: String,
    pub event_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Envelope<T> {
    pub schema_version: u32,
    pub repo_id: String,
    pub items: Vec<T>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
