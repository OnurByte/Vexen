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

/// Select at most three eligible lessons in search order, then reconcile their
/// outcome groups. Selection memory stays bounded while callers scan the store.
pub(crate) struct AdvisorySelection {
    records: Vec<MemoryRecord>,
    conflicted: Vec<bool>,
    limit: usize,
}

impl AdvisorySelection {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            records: Vec::new(),
            conflicted: Vec::new(),
            limit: limit.min(MAX_ITEMS),
        }
    }

    pub(crate) fn is_full(&self) -> bool {
        self.records.len() >= self.limit
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub(crate) fn needs_conflict_scan(&self) -> bool {
        self.records.iter().any(|r| r.kind == EventKind::Attempt)
    }

    pub(crate) fn consider(&mut self, record: &MemoryRecord, ctx: &RepoContext) {
        if self.is_full() {
            return;
        }
        let accepted = record.kind == EventKind::AcceptedDecision;
        if !accepted
            && (record.kind != EventKind::Attempt
                || record.status != RecordStatus::Observed
                || !record.outcomes.iter().any(|o| o.exit_code != 0))
        {
            return;
        }
        if !accepted
            && self
                .records
                .iter()
                .any(|r| r.kind == EventKind::Attempt && same_group(r, record))
        {
            return;
        }
        if check_applicability(record, ctx) != Eligibility::Eligible {
            return;
        }
        self.records.push(record.clone());
        self.conflicted.push(false);
    }

    pub(crate) fn observe_conflict(&mut self, record: &MemoryRecord, ctx: &RepoContext) {
        if record.kind != EventKind::Attempt
            || record.status != RecordStatus::Observed
            || !record.outcomes.iter().any(|o| o.exit_code == 0)
        {
            return;
        }
        for (i, selected) in self.records.iter().enumerate() {
            if !self.conflicted[i]
                && selected.kind == EventKind::Attempt
                && same_group(selected, record)
                && check_applicability(record, ctx) == Eligibility::Eligible
            {
                self.conflicted[i] = true;
            }
        }
    }

    pub(crate) fn finish(self) -> Vec<Advisory> {
        let selected = self
            .records
            .iter()
            .zip(self.conflicted)
            .map(|(record, conflict)| {
                let status = if record.kind == EventKind::AcceptedDecision {
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
                Advisory {
                    id: record.id.to_string(),
                    status,
                    text: format!(
                        "{qualifier} for {}: {}",
                        record.anchor_path,
                        quote_untrusted(&record.summary)
                    ),
                }
            })
            .collect();
        bound_output(selected)
    }
}

fn same_group(a: &MemoryRecord, b: &MemoryRecord) -> bool {
    a.anchor_path == b.anchor_path
        && a.anchor_sha256 == b.anchor_sha256
        && a.conditions == b.conditions
}

/// Filter and bound records, preserving search ordering. A passing attempt is
/// never used as the representative of a failed approach.
pub fn rank_applicable(records: &[MemoryRecord], ctx: &RepoContext, limit: usize) -> Vec<Advisory> {
    let mut selection = AdvisorySelection::new(limit);
    for record in records {
        selection.consider(record, ctx);
    }
    for record in records {
        selection.observe_conflict(record, ctx);
    }
    selection.finish()
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
