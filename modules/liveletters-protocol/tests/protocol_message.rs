use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolError, ProtocolMessage, decode_message,
    encode_message,
};

#[test]
fn post_created_round_trip_keeps_envelope_and_payload() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        "Новая запись в блоге",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            visibility: "public".into(),
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    assert_eq!(decoded.envelope().schema_version(), "1");
    assert_eq!(decoded.envelope().event_type(), "post_created");
    assert_eq!(decoded.envelope().resource_id(), "blog-1");
    assert_eq!(decoded.envelope().event_id(), "event-1");
    assert_eq!(decoded.human_readable_body(), "Новая запись в блоге");
    assert_eq!(decoded.payload(), message.payload());
}

#[test]
fn comment_created_round_trip_keeps_parent_comment_link() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_created", "blog-1", "event-2").unwrap(),
        "Новый комментарий",
        DomainEventPayload::CommentCreated {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: Some("comment-root".into()),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_100,
            visibility: "friends_only".into(),
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded.payload() {
        DomainEventPayload::CommentCreated {
            parent_comment_id, ..
        } => assert_eq!(parent_comment_id.as_deref(), Some("comment-root")),
        other => panic!("unexpected payload after decode: {other:?}"),
    }
}

#[test]
fn malformed_json_is_rejected() {
    let error = decode_message("{not-json").expect_err("malformed json must fail");

    assert!(matches!(error, ProtocolError::MalformedJson(_)));
}

#[test]
fn blank_human_body_is_rejected() {
    let error = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        "   ",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            visibility: "public".into(),
        },
    )
    .expect_err("blank human body must fail");

    assert_eq!(error, ProtocolError::BlankHumanReadableBody);
}

#[test]
fn post_hidden_round_trip_keeps_payload() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_hidden", "blog-1", "event-3").unwrap(),
        "Запись скрыта",
        DomainEventPayload::PostHidden {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_200,
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    assert_eq!(decoded.payload(), message.payload());
}

#[test]
fn comment_edited_round_trip_keeps_new_body() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_edited", "blog-1", "event-4").unwrap(),
        "Комментарий изменен",
        DomainEventPayload::CommentEdited {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_300,
            body: "Исправленный комментарий".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded.payload() {
        DomainEventPayload::CommentEdited { body, .. } => {
            assert_eq!(body, "Исправленный комментарий")
        }
        other => panic!("unexpected payload after decode: {other:?}"),
    }
}

#[test]
fn subscription_changed_subscribe_round_trip_keeps_payload() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_changed",
            "alice-publish@example.org",
            "sub-1",
        )
        .unwrap(),
        "Подписка",
        DomainEventPayload::SubscriptionChanged {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
            active: true,
            created_at: 1_710_000_400,
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded.payload() {
        DomainEventPayload::SubscriptionChanged {
            resource_address,
            subscriber_delivery_address,
            active,
            created_at,
        } => {
            assert_eq!(resource_address, "alice-publish@example.org");
            assert_eq!(subscriber_delivery_address, "bob-feed@example.org");
            assert!(*active);
            assert_eq!(*created_at, 1_710_000_400);
        }
        other => panic!("unexpected payload after decode: {other:?}"),
    }
}

#[test]
fn subscription_changed_serializes_with_snake_case_tag() {
    let payload = DomainEventPayload::SubscriptionChanged {
        resource_address: "alice-publish@example.org".into(),
        subscriber_delivery_address: "bob-feed@example.org".into(),
        active: false,
        created_at: 1_710_000_500,
    };

    let json = serde_json::to_value(&payload).unwrap();

    assert_eq!(json["kind"], "subscription_changed");
    assert_eq!(json["active"], false);
    assert!(json.get("subscriber_account_id").is_none());
}

#[test]
fn subscription_changed_decodes_legacy_account_id_but_ignores_it() {
    let json = r#"{
        "envelope": {
            "schema_version": "1",
            "event_type": "subscription_changed",
            "resource_id": "alice-publish@example.org",
            "event_id": "sub-legacy"
        },
        "human_readable_body": "Подписка",
        "payload": {
            "kind": "subscription_changed",
            "resource_address": "alice-publish@example.org",
            "subscriber_account_id": "acct_bob",
            "subscriber_delivery_address": "bob-feed@example.org",
            "action": "subscribe",
            "created_at": 1710000600
        }
    }"#;

    let decoded = decode_message(json).unwrap();

    assert_eq!(
        decoded.payload(),
        &DomainEventPayload::SubscriptionChanged {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
            active: true,
            created_at: 1_710_000_600,
        }
    );
}
