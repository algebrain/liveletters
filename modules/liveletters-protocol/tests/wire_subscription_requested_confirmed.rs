//! Сериализация и round-trip для трёх отдельных типов подписки:
//! `SubscriptionRequested`, `SubscriptionConfirmed`, `SubscriptionRevoked`.
//!
//! Все три не должны содержать внутренний служебный префикс `acct_` в
//! сериализованной форме. `SubscriptionConfirmed` дополнительно проверяется
//! на наличие профиля владельца блога.

use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage, encode_message,
};

fn json_of(payload: DomainEventPayload, origin: ProtocolIdentity) -> String {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "subscription_requested", "blog-1", "event-1").unwrap(),
        origin,
        None,
        "Тело",
        payload,
    )
    .unwrap();
    let json_via_message = serde_json::to_string(&message).unwrap();
    assert!(
        !json_via_message.contains("acct_"),
        "ProtocolMessage JSON не должен содержать acct_:\n{json_via_message}"
    );
    encode_message(&message).unwrap()
}

#[test]
fn subscription_requested_round_trip() {
    let payload = DomainEventPayload::SubscriptionRequested {
        resource_address: "alice@example.org".into(),
        subscriber_delivery_address: "bob@example.org".into(),
        created_at: 1_710_000_000,
    };
    let encoded = json_of(
        payload.clone(),
        ProtocolIdentity::new("Борис", "bob@example.org").unwrap(),
    );
    let json = serde_json::to_string(&payload).unwrap();
    assert!(
        json.contains("subscription_requested"),
        "JSON должен содержать kind=subscription_requested: {json}"
    );
    assert!(!json.contains("subscriber_nickname"), "json: {json}");
    let encoded_json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(encoded_json["origin"], "Борис <bob@example.org>");
    assert!(!json.contains("acct_"), "json: {json}");
    assert!(!encoded.contains("acct_"), "encoded: {encoded}");
}

#[test]
fn subscription_confirmed_uses_origin_for_owner_profile_and_accepted_flag() {
    let payload = DomainEventPayload::SubscriptionConfirmed {
        resource_address: "alice@example.org".into(),
        subscriber_delivery_address: "bob@example.org".into(),
        accepted: true,
        created_at: 1_710_000_000,
    };
    let encoded = json_of(
        payload.clone(),
        ProtocolIdentity::new("Алиса", "alice@example.org").unwrap(),
    );
    let payload_json = serde_json::to_string(&payload).unwrap();
    assert!(payload_json.contains("\"accepted\":true"));
    assert!(!payload_json.contains("owner_nickname"));
    assert!(!payload_json.contains("owner_email"));
    let encoded_json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(encoded_json["origin"], "Алиса <alice@example.org>");
    assert!(!encoded.contains("acct_"));
}

#[test]
fn subscription_confirmed_with_declined_keeps_profile() {
    let payload = DomainEventPayload::SubscriptionConfirmed {
        resource_address: "alice@example.org".into(),
        subscriber_delivery_address: "bob@example.org".into(),
        accepted: false,
        created_at: 1_710_000_000,
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"accepted\":false"));
    assert!(!json.contains("acct_"));
}

#[test]
fn subscription_revoked_round_trip() {
    let payload = DomainEventPayload::SubscriptionRevoked {
        resource_address: "alice@example.org".into(),
        subscriber_delivery_address: "bob@example.org".into(),
        created_at: 1_710_000_000,
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("subscription_revoked"));
    assert!(!json.contains("acct_"));
}
