#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncHealth {
    pub status: HealthStatus,
    pub applied_messages: usize,
    pub duplicate_messages: usize,
    pub malformed_messages: usize,
    pub deferred_events: usize,
    pub pending_outbox: usize,
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
    deferred_events: Vec<DeferredEventDiagnostic>,
    outbox_entries: Vec<OutboxDiagnostic>,
}

impl DiagnosticsSnapshot {
    pub fn new(
        sync_health: SyncHealth,
        raw_messages: Vec<RawMessageDiagnostic>,
        deferred_events: Vec<DeferredEventDiagnostic>,
        outbox_entries: Vec<OutboxDiagnostic>,
    ) -> Self {
        Self {
            sync_health,
            raw_messages,
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

    pub fn deferred_events(&self) -> &[DeferredEventDiagnostic] {
        &self.deferred_events
    }

    pub fn outbox_entries(&self) -> &[OutboxDiagnostic] {
        &self.outbox_entries
    }
}
