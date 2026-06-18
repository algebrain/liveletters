//! Идемпотентность протокольных событий: повторное применение с **новым**
//! `event_id`, но **тем же** логическим ключом не должно менять
//! материальное состояние БД.
//!
//! Шесть событий (`CommentCreated`, `PostHidden`, `CommentEdited`,
//! `SubscriptionRequested`, `SubscriptionConfirmed`, `SubscriptionRevoked`)
//! покрываются здесь. `PostCreated` уже покрыт в `sync_ingest.rs`.
//!
//! Допустимые расхождения между снимками БД до/после повтора:
//!
//! - `raw_messages` и `raw_events` — пополняются записями.
//! - `outbox` для `SubscriptionRequested` — может содержать две записи
//!   `SubscriptionConfirmed`; это сознательно (см. план).
//!
//! Материальные таблицы (`posts`, `comments`, `subscriptions`,
//! `local_subscriptions`, `pending_subscriptions`) — должны остаться
//! неизменными.

use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage,
};
use liveletters_store::{OutboxDelivery, OutboxRecord, Store};
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

mod common;

fn identity(nickname: &str, email: &str) -> ProtocolIdentity {
    ProtocolIdentity::new(nickname.to_owned(), email.to_owned()).unwrap()
}

fn build_email(
    event_id: &str,
    event_type: &str,
    resource_id: &str,
    origin_email: &str,
    payload: DomainEventPayload,
    human_body: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", event_type, resource_id, event_id).unwrap(),
        identity("Алиса", origin_email),
        None,
        human_body,
        payload,
    )
    .unwrap();
    let outgoing = build_protocol_email(
        origin_email,
        "receiver@example.test",
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

fn save_alice(store: &Store) {
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
}

fn save_bob(store: &Store) {
    store
        .save_identity("bob", "bob-publish@example.org", "Борис", None, "ru", true)
        .unwrap();
}

#[test]
fn comment_created_replay_does_not_change_db() {
    let (store, _tmp) = common::open_temp_store();
    store.save_author("blog-1", "blog", "self").unwrap();
    store
        .save_author("alice@example.test", "alice", "self")
        .unwrap();
    store
        .save_post_record(&liveletters_store::PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice@example.test".into(),
            created_at: 1,
            body: "Пост".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let payload = || DomainEventPayload::CommentCreated {
        comment_id: "comment-1".into(),
        post_id: "post-1".into(),
        parent_comment_id: None,
        resource_id: "blog-1".into(),
        created_at: 2,
        body: "Первый комментарий".into(),
        body_format: "plain".into(),
        visibility: "public".into(),
    };
    let first = build_email(
        "event-1",
        "comment_created",
        "blog-1",
        "alice@example.test",
        payload(),
        "Первый комментарий",
    );
    let second = build_email(
        "event-2",
        "comment_created",
        "blog-1",
        "alice@example.test",
        payload(),
        "Первый комментарий",
    );

    let report1 = engine.ingest_batch(vec![first]).expect("first ingest");
    let outcome1 = &report1.outcomes()[0];
    assert!(
        matches!(outcome1, SyncMessageOutcome::Applied { .. }),
        "первое скрытие должно примениться: {outcome1:?}"
    );
    let snapshot_before = store.list_comments_for_post("post-1").unwrap();
    assert_eq!(snapshot_before.len(), 1);
    assert_eq!(snapshot_before[0].body, "Первый комментарий");

    let report2 = engine.ingest_batch(vec![second]).expect("second ingest");
    assert!(matches!(
        report2.outcomes()[0],
        SyncMessageOutcome::Replay { .. }
    ));

    let snapshot_after = store.list_comments_for_post("post-1").unwrap();
    assert_eq!(
        snapshot_after, snapshot_before,
        "комментарий не должен измениться"
    );

    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events.len(), 2);
    assert_eq!(raw_events[0].apply_status, "applied");
    assert_eq!(raw_events[1].apply_status, "replay");
}

#[test]
fn post_hidden_replay_does_not_change_db() {
    let (store, _tmp) = common::open_temp_store();
    store.save_author("blog-1", "blog", "self").unwrap();
    store
        .save_author("alice@example.test", "alice", "self")
        .unwrap();
    store
        .save_post_record(&liveletters_store::PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice@example.test".into(),
            created_at: 1,
            body: "Пост".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let payload = || DomainEventPayload::PostHidden {
        post_id: "post-1".into(),
        resource_id: "blog-1".into(),
        created_at: 2,
    };
    let first = build_email(
        "event-1",
        "post_hidden",
        "blog-1",
        "alice@example.test",
        payload(),
        "Скрыто",
    );
    let second = build_email(
        "event-2",
        "post_hidden",
        "blog-1",
        "alice@example.test",
        payload(),
        "Скрыто",
    );

    let report1 = engine.ingest_batch(vec![first]).expect("first ingest");
    let outcome1 = &report1.outcomes()[0];
    assert!(
        matches!(outcome1, SyncMessageOutcome::Applied { .. }),
        "первое скрытие должно примениться: {outcome1:?}"
    );
    let snapshot_before = store.list_posts().unwrap();
    assert_eq!(snapshot_before.len(), 1);
    assert!(snapshot_before[0].hidden);

    let report2 = engine.ingest_batch(vec![second]).expect("second ingest");
    assert!(matches!(
        report2.outcomes()[0],
        SyncMessageOutcome::Replay { .. }
    ));

    let snapshot_after = store.list_posts().unwrap();
    assert_eq!(snapshot_after, snapshot_before, "пост не должен измениться");

    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events.len(), 2);
    assert_eq!(raw_events[0].apply_status, "applied");
    assert_eq!(raw_events[1].apply_status, "replay");
}

#[test]
fn comment_edited_replay_does_not_change_db() {
    let (store, _tmp) = common::open_temp_store();
    store.save_author("blog-1", "blog", "self").unwrap();
    store
        .save_author("alice@example.test", "alice", "self")
        .unwrap();
    store
        .save_post_record(&liveletters_store::PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice@example.test".into(),
            created_at: 1,
            body: "Пост".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    store
        .save_comment_record(&liveletters_store::CommentRecord {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: None,
            author_email: "alice@example.test".into(),
            created_at: 2,
            body: "Оригинал".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let payload = || DomainEventPayload::CommentEdited {
        comment_id: "comment-1".into(),
        post_id: "post-1".into(),
        resource_id: "blog-1".into(),
        created_at: 3,
        body: "Отредактировано".into(),
        visibility: "public".into(),
    };
    let first = build_email(
        "event-1",
        "comment_edited",
        "blog-1",
        "alice@example.test",
        payload(),
        "Отредактировано",
    );
    let second = build_email(
        "event-2",
        "comment_edited",
        "blog-1",
        "alice@example.test",
        payload(),
        "Отредактировано",
    );

    let report1 = engine.ingest_batch(vec![first]).expect("first ingest");
    assert!(matches!(
        report1.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    let snapshot_before = store.list_comments_for_post("post-1").unwrap();
    assert_eq!(snapshot_before[0].body, "Отредактировано");

    let report2 = engine.ingest_batch(vec![second]).expect("second ingest");
    assert!(matches!(
        report2.outcomes()[0],
        SyncMessageOutcome::Replay { .. }
    ));

    let snapshot_after = store.list_comments_for_post("post-1").unwrap();
    assert_eq!(
        snapshot_after, snapshot_before,
        "комментарий не должен измениться"
    );

    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events.len(), 2);
    assert_eq!(raw_events[0].apply_status, "applied");
    assert_eq!(raw_events[1].apply_status, "replay");
}

#[test]
fn subscription_requested_replay_keeps_subscriptions_table_stable() {
    let (store, _tmp) = common::open_temp_store();
    save_alice(&store);
    store
        .save_author("bob-feed@example.org", "Борис", "test")
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let payload = || DomainEventPayload::SubscriptionRequested {
        resource_id: "alice-publish@example.org".into(),
        subscriber_delivery_address: "bob-feed@example.org".into(),
        created_at: 1_710_000_000,
    };
    let first = build_email(
        "event-1",
        "subscription_requested",
        "alice-publish@example.org",
        "bob-feed@example.org",
        payload(),
        "Запрос подписки",
    );
    let second = build_email(
        "event-2",
        "subscription_requested",
        "alice-publish@example.org",
        "bob-feed@example.org",
        payload(),
        "Запрос подписки",
    );

    let report1 = engine.ingest_batch(vec![first]).expect("first ingest");
    assert!(matches!(
        report1.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    let subs_before = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(subs_before.len(), 1);

    let report2 = engine.ingest_batch(vec![second]).expect("second ingest");
    let outcome2 = &report2.outcomes()[0];
    assert!(
        matches!(outcome2, SyncMessageOutcome::Applied { .. }),
        "ожидался Applied (UPSERT), получили: {outcome2:?}"
    );

    let subs_after = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(
        subs_after, subs_before,
        "таблица subscriptions не должна расти при повторе"
    );

    // outbox может содержать две записи SubscriptionConfirmed — это сознательно.
    // Документируем наблюдаемое поведение.
    let outbox = store.list_outbox_records().unwrap();
    let confirmed_count = outbox
        .iter()
        .filter(|r| r.event_type == "subscription_confirmed")
        .count();
    assert!(
        confirmed_count >= 1,
        "в outbox должен быть хотя бы один SubscriptionConfirmed"
    );
}

#[test]
fn repeated_subscription_requested_keeps_response_semantically_same() {
    let (store, _tmp) = common::open_temp_store();
    save_alice(&store);
    store
        .save_author("bob-feed@example.org", "Борис", "test")
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let payload = || DomainEventPayload::SubscriptionRequested {
        resource_id: "alice-publish@example.org".into(),
        subscriber_delivery_address: "bob-feed@example.org".into(),
        created_at: 1_710_000_000,
    };
    let first = build_email(
        "event-1",
        "subscription_requested",
        "alice-publish@example.org",
        "bob-feed@example.org",
        payload(),
        "Запрос подписки",
    );
    let second = build_email(
        "event-2",
        "subscription_requested",
        "alice-publish@example.org",
        "bob-feed@example.org",
        payload(),
        "Запрос подписки",
    );

    engine.ingest_batch(vec![first]).expect("first ingest");
    let first_response = latest_subscription_confirmed(&store);
    let first_semantics = subscription_confirmed_semantics(&first_response);

    engine.ingest_batch(vec![second]).expect("second ingest");
    let second_response = latest_subscription_confirmed(&store);
    let second_semantics = subscription_confirmed_semantics(&second_response);

    assert_eq!(
        second_semantics, first_semantics,
        "повторный запрос должен давать такой же по смыслу ответ"
    );
}

#[test]
fn subscription_confirmed_accepted_replay_does_not_change_db() {
    let (store, _tmp) = common::open_temp_store();
    save_alice(&store);
    save_bob(&store);
    store
        .save_pending_subscription("bob", "alice-publish@example.org", 1_710_000_400)
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("bob");

    let payload = || DomainEventPayload::SubscriptionConfirmed {
        resource_id: "alice-publish@example.org".into(),
        subscriber_delivery_address: "bob-publish@example.org".into(),
        accepted: true,
        created_at: 1_710_000_500,
    };
    let first = build_email(
        "event-1",
        "subscription_confirmed",
        "alice-publish@example.org",
        "alice-publish@example.org",
        payload(),
        "Подтверждение",
    );
    let second = build_email(
        "event-2",
        "subscription_confirmed",
        "alice-publish@example.org",
        "alice-publish@example.org",
        payload(),
        "Подтверждение",
    );

    let report1 = engine.ingest_batch(vec![first]).expect("first ingest");
    let outcome1 = &report1.outcomes()[0];
    assert!(
        matches!(outcome1, SyncMessageOutcome::Applied { .. }),
        "первое подтверждение должно примениться: {outcome1:?}"
    );
    let local_before = store.list_local_subscriptions("bob").unwrap();
    let subs_before = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    let pending_before = store.list_pending_subscriptions("bob").unwrap();
    assert_eq!(local_before, vec!["alice-publish@example.org".to_owned()]);
    assert_eq!(pending_before.len(), 0);

    let report2 = engine.ingest_batch(vec![second]).expect("second ingest");
    let outcome2 = &report2.outcomes()[0];
    assert!(
        matches!(outcome2, SyncMessageOutcome::Applied { .. }),
        "повторное подтверждение должно тихо игнорироваться (Applied без эффекта): {outcome2:?}"
    );

    let local_after = store.list_local_subscriptions("bob").unwrap();
    let subs_after = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    let pending_after = store.list_pending_subscriptions("bob").unwrap();
    assert_eq!(
        local_after, local_before,
        "local_subscriptions не должны расти"
    );
    assert_eq!(subs_after, subs_before, "subscriptions не должны расти");
    assert_eq!(
        pending_after.len(),
        0,
        "pending_subscriptions остаются пустыми"
    );
}

#[derive(Debug, PartialEq, Eq)]
struct SubscriptionConfirmedSemantics {
    event_type: String,
    origin_nickname: String,
    origin_email: String,
    resource_id: String,
    subscriber_delivery_address: String,
    accepted: bool,
    delivery: OutboxDelivery,
    subject: Option<String>,
    human_readable_body: Option<String>,
}

fn latest_subscription_confirmed(store: &Store) -> OutboxRecord {
    store
        .list_outbox_records()
        .unwrap()
        .into_iter()
        .rev()
        .find(|record| record.event_type == "subscription_confirmed")
        .expect("subscription_confirmed должен быть в outbox")
}

fn subscription_confirmed_semantics(record: &OutboxRecord) -> SubscriptionConfirmedSemantics {
    let message = liveletters_protocol::decode_message(&record.message_body).unwrap();
    let DomainEventPayload::SubscriptionConfirmed {
        resource_id,
        subscriber_delivery_address,
        accepted,
        ..
    } = message.payload()
    else {
        panic!("ожидался SubscriptionConfirmed");
    };
    SubscriptionConfirmedSemantics {
        event_type: record.event_type.clone(),
        origin_nickname: message.origin().nickname().to_owned(),
        origin_email: message.origin().email().to_owned(),
        resource_id: resource_id.clone(),
        subscriber_delivery_address: subscriber_delivery_address.clone(),
        accepted: *accepted,
        delivery: record.delivery.clone(),
        subject: record.subject.clone(),
        human_readable_body: record.human_readable_body.clone(),
    }
}

#[test]
fn subscription_revoked_replay_does_not_change_db() {
    let (store, _tmp) = common::open_temp_store();
    save_alice(&store);
    save_bob(&store);
    store
        .save_subscription(&liveletters_store::SubscriptionRecord {
            resource_email: "alice-publish@example.org".into(),
            subscriber_email: "bob-publish@example.org".into(),
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("bob");

    let payload = || DomainEventPayload::SubscriptionRevoked {
        resource_id: "alice-publish@example.org".into(),
        subscriber_delivery_address: "bob-publish@example.org".into(),
        created_at: 1_710_001_000,
    };
    let first = build_email(
        "event-1",
        "subscription_revoked",
        "alice-publish@example.org",
        "alice-publish@example.org",
        payload(),
        "Отзыв",
    );
    let second = build_email(
        "event-2",
        "subscription_revoked",
        "alice-publish@example.org",
        "alice-publish@example.org",
        payload(),
        "Отзыв",
    );

    let report1 = engine.ingest_batch(vec![first]).expect("first ingest");
    let outcome1 = &report1.outcomes()[0];
    assert!(
        matches!(outcome1, SyncMessageOutcome::Applied { .. }),
        "первый отзыв должен примениться: {outcome1:?}"
    );
    let subs_before = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(subs_before.len(), 0);

    let report2 = engine.ingest_batch(vec![second]).expect("second ingest");
    let outcome2 = &report2.outcomes()[0];
    assert!(
        matches!(outcome2, SyncMessageOutcome::Applied { .. }),
        "повторный отзыв без записи должен тихо игнорироваться: {outcome2:?}"
    );

    let subs_after = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(subs_after, subs_before, "subscriptions не должны меняться");
}
