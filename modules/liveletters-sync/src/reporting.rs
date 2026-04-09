#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMessageOutcome {
    Applied { message_id: String, event_id: String },
    Duplicate { message_id: String, event_id: String },
    Replay { message_id: String, event_id: String, reason: String },
    Unauthorized { message_id: String, event_id: String, reason: String },
    Invalid { message_id: String, event_id: String, reason: String },
    Deferred { message_id: String, event_id: String, reason: String },
    Malformed { message_id: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    outcomes: Vec<SyncMessageOutcome>,
}

impl SyncReport {
    pub fn new(outcomes: Vec<SyncMessageOutcome>) -> Self {
        Self { outcomes }
    }

    pub fn outcomes(&self) -> &[SyncMessageOutcome] {
        &self.outcomes
    }
}
