use crate::{
    analysis::{Eligibility, RepoContext, check_applicability},
    privacy::redact_secrets,
    store::{EventKind, MemoryRecord, RecordStatus},
};

const MAX_ITEMS: usize = 3;
const MAX_TEXT_BYTES: usize = 1800;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisoryStatus {
    ObservedFailure,
    AcceptedDecision,
    Conflicted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Advisory {
    pub id: String,
    pub status: AdvisoryStatus,
    /// Plain prose only; imported record text is quoted as untrusted data.
    pub text: String,
}

/// Filter and bound records, preserving the Store search ordering (FTS rank).
/// Callers must supply records from the explicitly scoped Store search; ranking
/// is never allowed to promote an ineligible record.
pub fn rank_applicable(records: &[MemoryRecord], ctx: &RepoContext, limit: usize) -> Vec<Advisory> {
    if limit == 0 {
        return Vec::new();
    }
    let eligible: Vec<&MemoryRecord> = records
        .iter()
        .filter(|record| check_applicability(record, ctx) == Eligibility::Eligible)
        .collect();
    let mut selected = Vec::new();
    let mut seen_groups = Vec::<(String, String, Vec<String>)>::new();
    for record in eligible {
        // Event kind carries the authority boundary; projection status is
        // descriptive and cannot promote an attempt into a human decision.
        let accepted = record.kind == EventKind::AcceptedDecision;
        if !accepted && record.status != RecordStatus::Observed {
            continue;
        }
        if !accepted && record.outcomes.is_empty() {
            continue;
        }
        let group = (
            record.anchor_path.clone(),
            record.anchor_sha256.clone(),
            record.conditions.clone(),
        );
        if !accepted {
            if seen_groups.contains(&group) {
                continue;
            }
            seen_groups.push(group.clone());
        }
        let related: Vec<_> = records
            .iter()
            .filter(|other| {
                other.anchor_path == group.0
                    && other.anchor_sha256 == group.1
                    && other.conditions == group.2
                    && check_applicability(other, ctx) == Eligibility::Eligible
                    && other.kind == EventKind::Attempt
                    && other.status == RecordStatus::Observed
            })
            .collect();
        let has_failure = related
            .iter()
            .flat_map(|r| r.outcomes.iter())
            .any(|o| o.exit_code != 0);
        let has_success = related
            .iter()
            .flat_map(|r| r.outcomes.iter())
            .any(|o| o.exit_code == 0);
        let conflict = !accepted && has_failure && has_success;
        if !accepted && !has_failure {
            continue;
        }
        let status = if accepted {
            AdvisoryStatus::AcceptedDecision
        } else if conflict {
            AdvisoryStatus::Conflicted
        } else {
            AdvisoryStatus::ObservedFailure
        };
        let qualifier = match status {
            AdvisoryStatus::AcceptedDecision => {
                "Human-accepted decision (not a verified test result)"
            }
            AdvisoryStatus::ObservedFailure => {
                "Observed failure (applicability checked; not a guarantee)"
            }
            AdvisoryStatus::Conflicted => {
                "Conflicting observed outcomes; abstain from a definitive conclusion"
            }
        };
        let quoted = quote_untrusted(&record.summary);
        let text = format!("{qualifier} for {}: {quoted}", record.anchor_path);
        selected.push(Advisory {
            id: record.id.to_string(),
            status,
            text,
        });
        if selected.len() >= limit.min(MAX_ITEMS) {
            break;
        }
    }
    bound_output(selected)
}

fn quote_untrusted(summary: &str) -> String {
    let cleaned = summary
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>();
    let redacted = redact_secrets(&cleaned);
    let clipped = truncate_utf8(&redacted, 360);
    format!("\"{}\"", clipped.replace('"', "\\\""))
}

fn bound_output(mut items: Vec<Advisory>) -> Vec<Advisory> {
    let mut remaining = MAX_TEXT_BYTES;
    for item in &mut items {
        if item.text.len() > remaining {
            item.text = truncate_utf8(&item.text, remaining);
        }
        remaining = remaining.saturating_sub(item.text.len());
    }
    items.truncate(MAX_ITEMS);
    items
}

fn truncate_utf8(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let mut end = max.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}
