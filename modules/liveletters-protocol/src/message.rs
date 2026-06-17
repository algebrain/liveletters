use crate::{DomainEventPayload, MessageEnvelope, ProtocolError, ProtocolIdentity};
use serde::{Deserialize, Serialize};

/// Полное техническое сообщение LiveLetters.
///
/// `origin` всегда означает первичный источник события: автора поста,
/// автора комментария или владельца ответа на подписку. Получатель сохраняет
/// `origin` в `authors`.
///
/// `source` означает непосредственный источник доставки. Обычно он совпадает
/// с `origin` и не пишется в JSON. Он отличается при пересылке: если Алиса
/// комментирует пост Боба, а Боб рассылает комментарий Еве, то
/// `origin = Alice <alice@example.org>`, `source = Bob <bob@example.org>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    envelope: MessageEnvelope,
    /// Первичный источник события. Обязательное поле JSON.
    origin: ProtocolIdentity,
    /// Непосредственный источник доставки. Если отсутствует, равен `origin`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<ProtocolIdentity>,
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
    /// Создает сообщение и не сериализует `source`, если он равен `origin`.
    pub fn new(
        envelope: MessageEnvelope,
        origin: ProtocolIdentity,
        source: Option<ProtocolIdentity>,
        human_readable_body: &str,
        payload: DomainEventPayload,
    ) -> Result<Self, ProtocolError> {
        let trimmed = human_readable_body.trim();
        if trimmed.is_empty() {
            return Err(ProtocolError::BlankHumanReadableBody);
        }

        let source = source.filter(|source| source != &origin);
        Ok(Self {
            envelope,
            origin,
            source,
            human_readable_body: Some(trimmed.to_owned()),
            payload,
        })
    }

    pub fn envelope(&self) -> &MessageEnvelope {
        &self.envelope
    }

    /// Первичный источник события. Его получатель сохраняет в `authors`.
    pub fn origin(&self) -> &ProtocolIdentity {
        &self.origin
    }

    /// Непосредственный источник доставки, если он отличается от `origin`.
    pub fn source(&self) -> Option<&ProtocolIdentity> {
        self.source.as_ref()
    }

    /// Непосредственный источник доставки с учетом правила `source == origin`
    /// при отсутствии поля `source` в JSON.
    pub fn effective_source(&self) -> &ProtocolIdentity {
        self.source.as_ref().unwrap_or(&self.origin)
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
