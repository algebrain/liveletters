use liveletters_store::Store;

use crate::{
    DeferredEventDiagnostic, DiagnosticsError, DiagnosticsSnapshot, HealthStatus, OutboxDiagnostic,
    RawMessageDiagnostic, SyncHealth,
};

pub struct DiagnosticsReader<'a> {
    store: &'a Store,
}

impl<'a> DiagnosticsReader<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn build_snapshot(&self) -> Result<DiagnosticsSnapshot, DiagnosticsError> {
        let raw_messages = self.store.list_raw_message_records()?;
        let deferred_events = self.store.list_deferred_event_records()?;
        let outbox_entries = self.store.list_outbox_records()?;

        let applied_messages = raw_messages.iter().filter(|record| record.status == "applied").count();
        let duplicate_messages = raw_messages
            .iter()
            .filter(|record| record.status == "duplicate")
            .count();
        let malformed_messages = raw_messages
            .iter()
            .filter(|record| record.status == "malformed")
            .count();

        let sync_health = SyncHealth {
            status: if malformed_messages > 0 || !deferred_events.is_empty() {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            },
            applied_messages,
            duplicate_messages,
            malformed_messages,
            deferred_events: deferred_events.len(),
            pending_outbox: outbox_entries.len(),
        };

        Ok(DiagnosticsSnapshot::new(
            sync_health,
            raw_messages
                .into_iter()
                .map(|record| RawMessageDiagnostic {
                    message_id: record.message_id,
                    status: record.status,
                    preview: sanitize_preview(&record.raw_message),
                })
                .collect(),
            deferred_events
                .into_iter()
                .map(|record| DeferredEventDiagnostic {
                    event_id: record.event_id,
                    event_type: record.event_type,
                    reason: record.reason,
                })
                .collect(),
            outbox_entries
                .into_iter()
                .map(|record| OutboxDiagnostic {
                    event_id: record.event_id,
                    event_type: record.event_type,
                    resource_id: record.resource_id,
                    preview: sanitize_preview(&record.message_body),
                })
                .collect(),
        ))
    }
}

fn sanitize_preview(value: &str) -> String {
    let shortened = if value.chars().count() > 160 {
        value.chars().take(160).collect::<String>()
    } else {
        value.to_owned()
    };

    shortened
        .split_whitespace()
        .map(mask_email_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn mask_email_token(token: &str) -> String {
    let normalized = token.trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | ';'));
    if let Some((_, domain)) = normalized.split_once('@') {
        if domain.contains('.') {
            let masked = format!("***@{domain}");
            return token.replacen(normalized, &masked, 1);
        }
    }

    token.to_owned()
}
