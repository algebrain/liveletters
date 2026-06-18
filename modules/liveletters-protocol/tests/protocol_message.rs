use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolError, ProtocolIdentity, ProtocolMessage,
    decode_message, encode_message,
};

fn alice() -> ProtocolIdentity {
    ProtocolIdentity::new("Alice", "alice@example.org").unwrap()
}

#[test]
fn post_created_round_trip_keeps_envelope_and_payload() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        alice(),
        None,
        "Новая запись в блоге",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            created_at: 1_710_000_000,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert!(
        json["payload"].get("actor_id").is_none(),
        "actor_id должен передаваться через origin, а не payload: {encoded}"
    );
    let decoded = decode_message(&encoded).unwrap();

    assert_eq!(decoded.envelope().schema_version(), "1");
    assert_eq!(decoded.envelope().event_type(), "post_created");
    assert_eq!(decoded.envelope().resource_id(), "blog-1");
    assert_eq!(decoded.envelope().event_id(), "event-1");
    assert_eq!(
        decoded.origin().to_wire_string(),
        "Alice <alice@example.org>"
    );
    assert_eq!(
        decoded.effective_source().to_wire_string(),
        "Alice <alice@example.org>"
    );
    assert!(decoded.source().is_none());
    // human_readable_body намеренно не сериализуется в JSON
    // (см. message.rs: skip_serializing default), чтобы не дублировать
    // text/plain. После десериализации поле == None; тело хранится
    // отдельно — в `OutboxRecord.human_readable_body`.
    assert_eq!(decoded.human_readable_body(), None);
    assert_eq!(decoded.payload(), message.payload());
}

#[test]
fn comment_created_round_trip_keeps_parent_comment_link() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_created", "blog-1", "event-2").unwrap(),
        alice(),
        Some(ProtocolIdentity::new("Bob", "bob@example.org").unwrap()),
        "Новый комментарий",
        DomainEventPayload::CommentCreated {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: Some("comment-root".into()),
            resource_id: "blog-1".into(),
            created_at: 1_710_000_100,
            body: "Текст комментария".into(),
            body_format: "plain".into(),
            visibility: "friends_only".into(),
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert!(
        json["payload"].get("actor_id").is_none(),
        "actor_id должен передаваться через origin, а не payload: {encoded}"
    );
    let decoded = decode_message(&encoded).unwrap();

    assert_eq!(decoded.origin().email(), "alice@example.org");
    assert_eq!(decoded.effective_source().email(), "bob@example.org");
    match decoded.payload() {
        DomainEventPayload::CommentCreated {
            parent_comment_id, ..
        } => assert_eq!(parent_comment_id.as_deref(), Some("comment-root")),
        other => panic!("unexpected payload after decode: {other:?}"),
    }
}

#[test]
fn source_is_omitted_when_equal_to_origin_and_encoded_when_different() {
    let origin = ProtocolIdentity::new("Alice", "alice@example.org").unwrap();
    let payload = DomainEventPayload::PostHidden {
        post_id: "post-1".into(),
        resource_id: "blog-1".into(),
        created_at: 1,
    };
    let without_source = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_hidden", "blog-1", "event-source-1").unwrap(),
        origin.clone(),
        Some(origin.clone()),
        "Запись скрыта",
        payload.clone(),
    )
    .unwrap();
    let without_source_json = serde_json::to_string(&without_source).unwrap();
    assert!(without_source_json.contains("\"origin\":\"Alice <alice@example.org>\""));
    assert!(
        !without_source_json.contains("\"source\""),
        "{without_source_json}"
    );

    let with_source = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_hidden", "blog-1", "event-source-2").unwrap(),
        origin,
        Some(ProtocolIdentity::new("Bob", "bob@example.org").unwrap()),
        "Запись скрыта",
        payload,
    )
    .unwrap();
    let with_source_json = serde_json::to_string(&with_source).unwrap();
    assert!(with_source_json.contains("\"origin\":\"Alice <alice@example.org>\""));
    assert!(with_source_json.contains("\"source\":\"Bob <bob@example.org>\""));
    let decoded = decode_message(&with_source_json).unwrap();
    assert_eq!(decoded.effective_source().email(), "bob@example.org");
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
        alice(),
        None,
        "   ",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            created_at: 1_710_000_000,
            body: "Текст поста".into(),
            body_format: "plain".into(),
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
        alice(),
        None,
        "Запись скрыта",
        DomainEventPayload::PostHidden {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
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
        alice(),
        None,
        "Комментарий изменен",
        DomainEventPayload::CommentEdited {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
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
fn subscription_requested_round_trip_keeps_payload() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_requested",
            "alice-publish@example.org",
            "sub-1",
        )
        .unwrap(),
        ProtocolIdentity::new("Борис", "bob-feed@example.org").unwrap(),
        None,
        "Запрос подписки",
        DomainEventPayload::SubscriptionRequested {
            resource_id: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
            created_at: 1_710_000_400,
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    assert_eq!(decoded.origin().nickname(), "Борис");
    assert_eq!(decoded.origin().email(), "bob-feed@example.org");
    match decoded.payload() {
        DomainEventPayload::SubscriptionRequested {
            resource_id,
            subscriber_delivery_address,
            created_at,
        } => {
            assert_eq!(resource_id, "alice-publish@example.org");
            assert_eq!(subscriber_delivery_address, "bob-feed@example.org");
            assert_eq!(*created_at, 1_710_000_400);
        }
        other => panic!("unexpected payload after decode: {other:?}"),
    }
}

#[test]
fn subscription_confirmed_round_trip_keeps_payload() {
    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_confirmed",
            "alice-publish@example.org",
            "sub-2",
        )
        .unwrap(),
        ProtocolIdentity::new("Алиса", "alice-publish@example.org").unwrap(),
        None,
        "Подтверждение",
        DomainEventPayload::SubscriptionConfirmed {
            resource_id: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
            accepted: true,
            created_at: 1_710_000_500,
        },
    )
    .unwrap();

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    assert_eq!(decoded.origin().nickname(), "Алиса");
    assert_eq!(decoded.origin().email(), "alice-publish@example.org");
    match decoded.payload() {
        DomainEventPayload::SubscriptionConfirmed {
            resource_id,
            subscriber_delivery_address,
            accepted,
            created_at,
        } => {
            assert_eq!(resource_id, "alice-publish@example.org");
            assert_eq!(subscriber_delivery_address, "bob-feed@example.org");
            assert!(*accepted);
            assert_eq!(*created_at, 1_710_000_500);
        }
        other => panic!("unexpected payload after decode: {other:?}"),
    }
}

#[test]
fn subscription_revoked_serializes_with_snake_case_tag() {
    let payload = DomainEventPayload::SubscriptionRevoked {
        resource_id: "alice-publish@example.org".into(),
        subscriber_delivery_address: "bob-feed@example.org".into(),
        created_at: 1_710_000_500,
    };

    let json = serde_json::to_value(&payload).unwrap();

    assert_eq!(json["kind"], "subscription_revoked");
    assert_eq!(json["resource_id"], "alice-publish@example.org");
    assert!(
        json.get("resource_address").is_none(),
        "subscription payload должен использовать resource_id: {json}"
    );
}
