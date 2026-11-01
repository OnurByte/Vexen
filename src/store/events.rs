use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable identifier for an event in one checkout's append-only log.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct EventId(pub(crate) String);

impl EventId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for EventId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Distinguishes a recorded attempt from a human-accepted decision.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Attempt,
    AcceptedDecision,
    Outcome,
    Supersession,
}

/// Input for a new immutable event. `anchor_path` is repo-relative and
/// `anchor_sha256` hashes the working-file contents at the recorded state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub struct NewEvent {
    pub idempotency_key: String,
    pub kind: EventKind,
    pub summary: String,
    pub anchor_path: String,
    pub anchor_sha256: String,
    pub git_commit: Option<String>,
    pub conditions: Vec<String>,
}

/// Allowlisted observed test metadata; raw process output is never accepted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub struct NewOutcome {
    pub idempotency_key: String,
    pub test_id: String,
    pub exit_code: i32,
    pub output_sha256: Option<String>,
}

/// Persisted event and its deterministic hash-chain metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub struct Event {
    pub id: EventId,
    pub parent_id: Option<EventId>,
    pub idempotency_key: String,
    pub sequence: u64,
    pub kind: EventKind,
    pub payload: EventPayload,
    pub previous_hash: String,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    Record(NewEvent),
    Outcome(NewOutcome),
    Supersession {
        idempotency_key: String,
        reason: String,
    },
}

/// A root event and any outcome events appended to it, in sequence order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub struct EventChain {
    pub events: Vec<Event>,
}

/// Search projection. `status` is descriptive evidence state, never a claim
/// that the validity/applicability layer has rechecked the anchor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MemoryRecord {
    pub repo_id: String,
    pub id: EventId,
    pub kind: EventKind,
    pub summary: String,
    pub anchor_path: String,
    pub anchor_sha256: String,
    pub git_commit: Option<String>,
    pub conditions: Vec<String>,
    pub status: RecordStatus,
    pub outcomes: Vec<NewOutcome>,
}

/// Evidence maturity in the projection; `observed` is not a validity claim.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    Candidate,
    AcceptedDecision,
    Observed,
    Superseded,
}
