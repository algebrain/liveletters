//! Гарантия того, что сериализованный JSON полезной нагрузки не содержит
//! внутренний служебный префикс `acct_`. В `actor_id` и `subscriber_*` поля
//! передаётся почтовый адрес или пустая строка; никаких `acct_<имя>`.

use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage, encode_message,
};

fn json_of(payload: DomainEventPayload) -> String {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        ProtocolIdentity::new("Alice", "alice@example.org").unwrap(),
        None,
        "Тело",
        payload,
    )
    .unwrap();
    let encoded = encode_message(&message).unwrap();
    // `encoded` — текстовое представление протокольного сообщения; проверяем,
    // что в нём нет `acct_`. Если в будущем формат изменится и `encoded`
    // перестанет содержать JSON-дословно, достаточно убедиться, что
    // сериализация payload'а в JSON не содержит `acct_`. Здесь мы проверяем
    // и то, и другое: и финальный encoded, и промежуточный JSON.
    let json_via_message = serde_json::to_string(&message).unwrap();
    assert!(
        !json_via_message.contains("acct_"),
        "ProtocolMessage JSON не должен содержать acct_:\n{json_via_message}"
    );
    encoded
}

#[test]
fn post_created_actor_id_email_does_not_leak_acct_prefix() {
    let payload = DomainEventPayload::PostCreated {
        post_id: "post-1".into(),
        resource_id: "blog-1".into(),
        actor_id: "alice@example.org".into(),
        created_at: 1_710_000_000,
        body: "Текст".into(),
        body_format: "plain".into(),
        visibility: "public".into(),
    };
    let encoded = json_of(payload);
    assert!(!encoded.contains("acct_"), "encoded: {encoded}");
}

#[test]
fn comment_created_actor_id_email_does_not_leak_acct_prefix() {
    let payload = DomainEventPayload::CommentCreated {
        comment_id: "c-1".into(),
        post_id: "post-1".into(),
        parent_comment_id: None,
        resource_id: "blog-1".into(),
        actor_id: "bob@example.org".into(),
        created_at: 1_710_000_000,
        body: "Текст".into(),
        body_format: "plain".into(),
        visibility: "public".into(),
    };
    let encoded = json_of(payload);
    assert!(!encoded.contains("acct_"), "encoded: {encoded}");
}

#[test]
fn subscription_requested_does_not_leak_acct_prefix() {
    let payload = DomainEventPayload::SubscriptionRequested {
        resource_address: "blog-1".into(),
        subscriber_delivery_address: "carol@example.org".into(),
        created_at: 1_710_000_000,
    };
    let encoded = json_of(payload);
    assert!(!encoded.contains("acct_"), "encoded: {encoded}");
}

#[test]
fn subscription_confirmed_does_not_leak_acct_prefix() {
    let payload = DomainEventPayload::SubscriptionConfirmed {
        resource_address: "blog-1".into(),
        subscriber_delivery_address: "carol@example.org".into(),
        accepted: true,
        created_at: 1_710_000_000,
    };
    let encoded = json_of(payload);
    assert!(!encoded.contains("acct_"), "encoded: {encoded}");
}
