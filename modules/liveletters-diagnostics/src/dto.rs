#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncHealth {
    status: HealthStatus,
    applied_messages: usize,
    duplicate_messages: usize,
    replayed_messages: usize,
    unauthorized_messages: usize,
    invalid_messages: usize,
    malformed_messages: usize,
    deferred_events: usize,
    pending_outbox: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncHealthFields {
    pub status: HealthStatus,
    pub applied_messages: usize,
    pub duplicate_messages: usize,
    pub replayed_messages: usize,
    pub unauthorized_messages: usize,
    pub invalid_messages: usize,
    pub malformed_messages: usize,
    pub deferred_events: usize,
    pub pending_outbox: usize,
}

impl SyncHealth {
    pub fn new(fields: SyncHealthFields) -> Self {
        Self {
            status: fields.status,
            applied_messages: fields.applied_messages,
            duplicate_messages: fields.duplicate_messages,
            replayed_messages: fields.replayed_messages,
            unauthorized_messages: fields.unauthorized_messages,
            invalid_messages: fields.invalid_messages,
            malformed_messages: fields.malformed_messages,
            deferred_events: fields.deferred_events,
            pending_outbox: fields.pending_outbox,
        }
    }

    pub fn status(&self) -> &HealthStatus {
        &self.status
    }
    pub fn applied_messages(&self) -> usize {
        self.applied_messages
    }
    pub fn duplicate_messages(&self) -> usize {
        self.duplicate_messages
    }
    pub fn replayed_messages(&self) -> usize {
        self.replayed_messages
    }
    pub fn unauthorized_messages(&self) -> usize {
        self.unauthorized_messages
    }
    pub fn invalid_messages(&self) -> usize {
        self.invalid_messages
    }
    pub fn malformed_messages(&self) -> usize {
        self.malformed_messages
    }
    pub fn deferred_events(&self) -> usize {
        self.deferred_events
    }
    pub fn pending_outbox(&self) -> usize {
        self.pending_outbox
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessageDiagnostic {
    pub message_id: String,
    pub status: String,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredEventDiagnostic {
    pub event_id: String,
    pub event_type: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEventDiagnostic {
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub apply_status: String,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxDiagnostic {
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsSnapshot {
    sync_health: SyncHealth,
    raw_messages: Vec<RawMessageDiagnostic>,
    raw_events: Vec<RawEventDiagnostic>,
    deferred_events: Vec<DeferredEventDiagnostic>,
    outbox_entries: Vec<OutboxDiagnostic>,
}

impl DiagnosticsSnapshot {
    pub fn new(
        sync_health: SyncHealth,
        raw_messages: Vec<RawMessageDiagnostic>,
        raw_events: Vec<RawEventDiagnostic>,
        deferred_events: Vec<DeferredEventDiagnostic>,
        outbox_entries: Vec<OutboxDiagnostic>,
    ) -> Self {
        Self {
            sync_health,
            raw_messages,
            raw_events,
            deferred_events,
            outbox_entries,
        }
    }

    pub fn sync_health(&self) -> &SyncHealth {
        &self.sync_health
    }

    pub fn raw_messages(&self) -> &[RawMessageDiagnostic] {
        &self.raw_messages
    }

    pub fn raw_events(&self) -> &[RawEventDiagnostic] {
        &self.raw_events
    }

    pub fn deferred_events(&self) -> &[DeferredEventDiagnostic] {
        &self.deferred_events
    }

    pub fn outbox_entries(&self) -> &[OutboxDiagnostic] {
        &self.outbox_entries
    }
}
