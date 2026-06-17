mod common;

use common::open_temp_store;
use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage,
};
use liveletters_store::SubscriptionRecord;
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

fn ensure_author(store: &liveletters_store::Store, email: &str, nickname: &str) {
    store.save_author(email, nickname, "test").unwrap();
}

fn identity(nickname: &str, email: &str) -> ProtocolIdentity {
    ProtocolIdentity::new(nickname.to_owned(), email.to_owned()).unwrap()
}

fn subscription_requested_email(
    event_id: &str,
    resource_address: &str,
    subscriber_delivery_address: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "subscription_requested", resource_address, event_id).unwrap(),
        identity("Test User", subscriber_delivery_address),
        None,
        "Запрос подписки",
        DomainEventPayload::SubscriptionRequested {
            resource_address: resource_address.into(),
            subscriber_delivery_address: subscriber_delivery_address.into(),
            created_at: 1_710_000_000,
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "bob@example.test",
        resource_address,
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();

    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

fn subscription_revoked_email(
    event_id: &str,
    resource_address: &str,
    subscriber_delivery_address: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "subscription_revoked", resource_address, event_id).unwrap(),
        identity("Test User", subscriber_delivery_address),
        None,
        "Отписка",
        DomainEventPayload::SubscriptionRevoked {
            resource_address: resource_address.into(),
            subscriber_delivery_address: subscriber_delivery_address.into(),
            created_at: 1_710_000_000,
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "bob@example.test",
        resource_address,
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();

    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

fn post_created_email(event_id: &str, post_id: &str, resource_id: &str) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", resource_id, event_id).unwrap(),
        identity("Alice", resource_id),
        None,
        "Новая запись",
        DomainEventPayload::PostCreated {
            post_id: post_id.into(),
            resource_id: resource_id.into(),
            actor_id: "alice".into(),
            created_at: 1,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();

    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

fn comment_created_email(
    event_id: &str,
    comment_id: &str,
    post_id: &str,
    resource_id: &str,
    author_email: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_created", resource_id, event_id).unwrap(),
        identity(author_email, author_email),
        None,
        "Новый комментарий",
        DomainEventPayload::CommentCreated {
            comment_id: comment_id.into(),
            post_id: post_id.into(),
            parent_comment_id: None,
            resource_id: resource_id.into(),
            actor_id: author_email.into(),
            created_at: 1,
            body: "Текст комментария".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        author_email,
        resource_id,
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();

    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

#[test]
fn apply_subscription_requested_persists_subscriber_for_redistribution() {
    let (store, _tmp) = open_temp_store();
    // A должен иметь user_settings, чтобы отправить SubscriptionConfirmed.
    store
        .save_identity(
            "alice",
            "alice-publish@example.org",
            "Алиса",
            None,
            "ru",
            true,
        )
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let report = engine
        .ingest_batch(vec![subscription_requested_email(
            "sub-1",
            "alice-publish@example.org",
            "bob-feed@example.org",
        )])
        .unwrap();

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));

    // A фиксирует подписку в своей БД — иначе у неё нет списка
    // адресатов для пересылки (PostCreated/CommentCreated).
    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(
        records.len(),
        1,
        "SubscriptionRequested должен фиксировать подписчика в subscriptions \
         (нужно для пересылки в ResourceSubscribers)"
    );
    assert_eq!(records[0].subscriber_email, "bob-feed@example.org");
}

#[test]
fn apply_subscription_confirmed_does_not_persist_record_yet() {
    // До этапа 5 SubscriptionConfirmed no-op; полное поведение в этапе 5.
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);
    let message = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_confirmed",
            "alice-publish@example.org",
            "sub-2",
        )
        .unwrap(),
        identity("Алиса", "alice-publish@example.org"),
        None,
        "Подтверждение",
        DomainEventPayload::SubscriptionConfirmed {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
            accepted: true,
            created_at: 1_710_000_000,
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice-publish@example.org",
        "bob-feed@example.org",
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();

    let report = engine
        .ingest_batch(vec![ReceivedEmail {
            message_id: "message-sub-2".into(),
            raw_message: outgoing.raw_message,
        }])
        .unwrap();
    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert!(records.is_empty());
}

#[test]
fn apply_subscription_revoked_is_noop_when_not_yet_subscribed() {
    // SubscriptionRevoked удаляет запись, но если её нет — no-op.
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![subscription_revoked_email(
            "unsub-1",
            "alice-publish@example.org",
            "bob-feed@example.org",
        )])
        .unwrap();
    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
}

#[test]
fn post_created_is_filtered_when_not_subscribed() {
    let (store, _tmp) = open_temp_store();
    let subscribed = vec!["carol-publish@example.org".to_string()];
    let engine = SyncEngine::new_with_identity(&store, "bob-publish@example.org", &subscribed);

    let report = engine
        .ingest_batch(vec![post_created_email(
            "post-alice-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Filtered { .. }
    ));
    assert!(store.list_posts().unwrap().is_empty());
    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events[0].apply_status, "filtered");
}

#[test]
fn post_created_is_applied_when_subscribed() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "alice-publish@example.org", "Alice");
    let subscribed = vec!["alice-publish@example.org".to_string()];
    let engine = SyncEngine::new_with_identity(&store, "bob-publish@example.org", &subscribed);

    let report = engine
        .ingest_batch(vec![post_created_email(
            "post-alice-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    assert_eq!(store.list_posts().unwrap().len(), 1);
}

#[test]
fn post_created_is_applied_when_own_resource() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "alice-publish@example.org", "Alice");
    let engine = SyncEngine::new_with_identity(&store, "alice-publish@example.org", &[]);

    let report = engine
        .ingest_batch(vec![post_created_email(
            "post-alice-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    assert_eq!(store.list_posts().unwrap().len(), 1);
}

#[test]
fn post_created_is_applied_when_engine_has_no_identity_filter() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "alice-publish@example.org", "Alice");
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![post_created_email(
            "post-alice-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
}

#[test]
fn applied_comment_created_creates_outbox_redistribution_to_other_subscribers() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "alice-publish@example.org", "Alice");
    ensure_author(&store, "bob@example.org", "Bob");
    ensure_author(&store, "eve@example.org", "Eve");
    let engine = SyncEngine::new(&store);

    // пост от alice в её блог
    let _ = engine
        .ingest_batch(vec![post_created_email(
            "post-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    // bob и eve — подписчики blog-1 (alice-publish)
    store
        .save_subscription(&SubscriptionRecord {
            resource_email: "alice-publish@example.org".into(),
            subscriber_email: "bob@example.org".into(),
        })
        .unwrap();
    store
        .save_subscription(&SubscriptionRecord {
            resource_email: "alice-publish@example.org".into(),
            subscriber_email: "eve@example.org".into(),
        })
        .unwrap();

    // bob комментирует (actor_id = его email — это важно для фильтра)
    let report = engine
        .ingest_batch(vec![comment_created_email(
            "comment-1",
            "comment-1",
            "post-1",
            "alice-publish@example.org",
            "bob@example.org",
        )])
        .unwrap();
    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));

    // alice должна положить outbox-запись пересылки
    let outbox = store.list_outbox_records().unwrap();
    let redist: Vec<_> = outbox
        .iter()
        .filter(|r| r.event_id.starts_with("redistribute:"))
        .collect();
    assert_eq!(redist.len(), 1, "ожидалась ровно 1 запись пересылки");
    match &redist[0].delivery {
        liveletters_store::OutboxDelivery::Direct(addrs) => {
            assert_eq!(addrs, &vec!["eve@example.org".to_owned()]);
        }
        other => panic!("ожидался Direct, получили {other:?}"),
    }

    // event_type — технический, subject — локализованная строка
    assert_eq!(
        redist[0].event_type, "comment_created",
        "event_type пересылки должен быть техническим, получили {:?}",
        redist[0].event_type
    );
    let subject = redist[0]
        .subject
        .as_deref()
        .expect("redistribute должен иметь subject");
    assert!(
        subject.contains("Новый комментарий"),
        "subject должен быть локализован, получили {subject:?}"
    );
    // Тело теперь хранится в отдельной колонке outbox
    // (OutboxRecord.human_readable_body), а не в JSON. Проверяем, что
    // локализованная подстановка с автором и post_id сохранена.
    let body = redist[0]
        .human_readable_body
        .as_deref()
        .expect("redistribute должен сохранить тело в отдельной колонке outbox");
    assert!(body.contains("bob"), "body должен содержать автора: {body}");
    assert!(
        body.contains("post-1"),
        "body должен содержать post_id: {body}"
    );
}

#[test]
fn comment_by_only_subscriber_creates_no_outbox_redistribution() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "alice-publish@example.org", "Alice");
    ensure_author(&store, "bob@example.org", "Bob");
    let engine = SyncEngine::new(&store);

    let _ = engine
        .ingest_batch(vec![post_created_email(
            "post-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    // bob — единственный подписчик, и он же автор комментария
    store
        .save_subscription(&SubscriptionRecord {
            resource_email: "alice-publish@example.org".into(),
            subscriber_email: "bob@example.org".into(),
        })
        .unwrap();

    let _ = engine
        .ingest_batch(vec![comment_created_email(
            "comment-1",
            "comment-1",
            "post-1",
            "alice-publish@example.org",
            "bob@example.org",
        )])
        .unwrap();

    let outbox = store.list_outbox_records().unwrap();
    assert!(
        outbox.is_empty(),
        "пересылка не нужна, если подписчик == автор: {outbox:?}"
    );
}
