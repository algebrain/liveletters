use crate::ProtocolError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageEnvelope {
    schema_version: String,
    event_type: String,
    resource_id: String,
    event_id: String,
}

fn require_non_blank(value: &str, field: &'static str) -> Result<String, ProtocolError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProtocolError::BlankEnvelopeField(field));
    }

    Ok(trimmed.to_owned())
}

impl MessageEnvelope {
    pub fn new(
        schema_version: &str,
        event_type: &str,
        resource_id: &str,
        event_id: &str,
    ) -> Result<Self, ProtocolError> {
        Ok(Self {
            schema_version: require_non_blank(schema_version, "schema_version")?,
            event_type: require_non_blank(event_type, "event_type")?,
            resource_id: require_non_blank(resource_id, "resource_id")?,
            event_id: require_non_blank(event_id, "event_id")?,
        })
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }
}
