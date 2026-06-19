use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage, decode_message,
    encode_message,
};

#[test]
fn friend_added_round_trip_has_no_subscription_purpose() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "friend_added",
            "alice@example.org",
            "friend-added:alice:bob",
        )
        .unwrap(),
        ProtocolIdentity::new("Alice", "alice@example.org").unwrap(),
        None,
        "Alice добавил(а) Вас в друзья",
        DomainEventPayload::FriendAdded {
            resource_id: "alice@example.org".into(),
            friend_address: "bob@example.org".into(),
            created_at: 1_770_000_000,
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(json["payload"]["kind"], "friend_added");
    assert!(json["payload"].get("purpose").is_none(), "{encoded}");

    let decoded = decode_message(&encoded).unwrap();
    assert_eq!(decoded.envelope().event_type(), "friend_added");
    assert_eq!(decoded.payload(), message.payload());
}
