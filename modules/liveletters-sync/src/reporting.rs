#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMessageOutcome {
    Applied {
        message_id: String,
        event_id: String,
    },
    Duplicate {
        message_id: String,
        event_id: String,
    },
    Replay {
        message_id: String,
        event_id: String,
        reason: String,
    },
    Unauthorized {
        message_id: String,
        event_id: String,
        reason: String,
    },
    Invalid {
        message_id: String,
        event_id: String,
        reason: String,
    },
    Deferred {
        message_id: String,
        event_id: String,
        reason: String,
    },
    Filtered {
        message_id: String,
        event_id: String,
        reason: String,
    },
    Malformed {
        message_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    outcomes: Vec<SyncMessageOutcome>,
    applied: usize,
    bounced: usize,
    pending_remaining: usize,
}

impl SyncReport {
    pub fn new(outcomes: Vec<SyncMessageOutcome>) -> Self {
        let applied = outcomes
            .iter()
            .filter(|o| matches!(o, SyncMessageOutcome::Applied { .. }))
            .count();
        let bounced = outcomes
            .iter()
            .filter(|o| {
                if let SyncMessageOutcome::Filtered { reason, .. } = o {
                    reason.starts_with("DSN") || reason.contains("bounce")
                } else {
                    false
                }
            })
            .count();
        Self {
            outcomes,
            applied,
            bounced,
            pending_remaining: 0,
        }
    }

    pub fn with_pending_remaining(mut self, n: usize) -> Self {
        self.pending_remaining = n;
        self
    }

    pub fn outcomes(&self) -> &[SyncMessageOutcome] {
        &self.outcomes
    }

    pub fn applied(&self) -> usize {
        self.applied
    }

    pub fn bounced(&self) -> usize {
        self.bounced
    }

    pub fn pending_remaining(&self) -> usize {
        self.pending_remaining
    }
}
