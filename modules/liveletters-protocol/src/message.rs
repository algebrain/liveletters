use crate::{DomainEventPayload, MessageEnvelope, ProtocolError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    envelope: MessageEnvelope,
    human_readable_body: String,
    payload: DomainEventPayload,
}

impl ProtocolMessage {
    pub fn new(
        envelope: MessageEnvelope,
        human_readable_body: &str,
        payload: DomainEventPayload,
    ) -> Result<Self, ProtocolError> {
        let trimmed = human_readable_body.trim();
        if trimmed.is_empty() {
            return Err(ProtocolError::BlankHumanReadableBody);
        }

        Ok(Self {
            envelope,
            human_readable_body: trimmed.to_owned(),
            payload,
        })
    }

    pub fn envelope(&self) -> &MessageEnvelope {
        &self.envelope
    }

    pub fn human_readable_body(&self) -> &str {
        &self.human_readable_body
    }

    pub fn payload(&self) -> &DomainEventPayload {
        &self.payload
    }
}
