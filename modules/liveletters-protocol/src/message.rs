use crate::{DomainEventPayload, MessageEnvelope, ProtocolError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    envelope: MessageEnvelope,
    // Локализованное тело письма. Намеренно не сериализуется в JSON
    // (помечено `skip_serializing`): в wire-формате строка дублировала
    // бы text/plain под-часть. Хранится отдельно — в
    // `OutboxRecord.human_readable_body` — и кладётся в email при
    // сборке. Поле в самой структуре сохранено: оно используется в
    // unit-тестах и на стороне отправителя, если тот держит
    // `ProtocolMessage` в памяти до сборки письма.
    //
    // `default` нужен, чтобы старые JSON без поля (или будущие,
    // где поля не будет) десериализовались в `None`, а не падали.
    #[serde(default, skip_serializing)]
    human_readable_body: Option<String>,
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
            human_readable_body: Some(trimmed.to_owned()),
            payload,
        })
    }

    pub fn envelope(&self) -> &MessageEnvelope {
        &self.envelope
    }

    /// Локализованное тело, сохранённое в самой структуре. Может быть
    /// `None`, если сообщение десериализовано из JSON, где поля нет.
    pub fn human_readable_body(&self) -> Option<&str> {
        self.human_readable_body.as_deref()
    }

    pub fn payload(&self) -> &DomainEventPayload {
        &self.payload
    }
}
