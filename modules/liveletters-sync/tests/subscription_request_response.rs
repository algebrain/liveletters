//! Sync: A автоотвечает `SubscriptionConfirmed` на `SubscriptionRequested`,
//! B применяет подтверждение (pending → subscriptions + local + display_names).

use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};
use liveletters_store::{Store, UserSettingsRecord};
use liveletters_sync::SyncEngine;

mod common;

fn open() -> (Store, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    (store, tmp)
}

#[allow(dead_code)]
fn _open_temp_store() -> (Store, tempfile::TempDir) {
    common::open_temp_store()
}

fn save_alice(store: &Store) {
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "alice".into(),
            nickname: "Алиса".into(),
            email_address: "alice-publish@example.org".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
}

fn build_subscription_requested_email(from: &str, to: &str, event_id: &str) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "subscription_requested", to, event_id).unwrap(),
        "Запрос подписки",
        DomainEventPayload::SubscriptionRequested {
            resource_address: to.into(),
            subscriber_delivery_address: from.into(),
            created_at: 1_710_000_000,
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        from,
        to,
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

fn build_subscription_confirmed_email(
    from: &str,
    to: &str,
    event_id: &str,
    accepted: bool,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "subscription_confirmed", from, event_id).unwrap(),
        if accepted {
            "Подтверждение"
        } else {
            "Отказ"
        },
        DomainEventPayload::SubscriptionConfirmed {
            resource_address: from.into(),
            subscriber_delivery_address: to.into(),
            owner_nickname: "Алиса".into(),
            owner_email: from.into(),
            accepted,
            created_at: 1_710_000_500,
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        from,
        to,
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
fn a_responds_to_subscription_request_with_confirmed() {
    let (store, _tmp) = open();
    save_alice(&store);
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let raw = build_subscription_requested_email(
        "bob-feed@example.org",
        "alice-publish@example.org",
        "sub-1",
    );

    let report = engine.ingest_batch(vec![raw]).expect("ingest ok");
    assert!(matches!(
        report.outcomes()[0],
        liveletters_sync::SyncMessageOutcome::Applied { .. }
    ));

    // A положил SubscriptionConfirmed в outbox
    let outbox = store.list_outbox_records().unwrap();
    let resp = outbox
        .iter()
        .find(|r| r.event_type == "subscription_confirmed")
        .expect("SubscriptionConfirmed в outbox");
    assert!(resp.message_id.is_some());
    let decoded = liveletters_protocol::decode_message(&resp.message_body).unwrap();
    match decoded.payload() {
        DomainEventPayload::SubscriptionConfirmed {
            owner_nickname,
            owner_email,
            accepted,
            ..
        } => {
            assert_eq!(owner_nickname, "Алиса");
            assert_eq!(owner_email, "alice-publish@example.org");
            assert!(accepted);
        }
        other => panic!("ожидался SubscriptionConfirmed, получили: {other:?}"),
    }
    // subject должен быть локализован на языке отправителя (alice, ru).
    let subject = resp
        .subject
        .as_deref()
        .expect("SubscriptionConfirmed должен иметь локализованный subject");
    assert!(
        subject.contains("Подписка подтверждена"),
        "subject должен быть на языке отправителя, получили: {subject:?}"
    );
}

#[test]
fn auto_confirm_uses_sender_language_for_subject() {
    let (store, _tmp) = open();
    // alice настроена на английский
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "alice".into(),
            nickname: "Alice".into(),
            email_address: "alice-publish@example.org".into(),
            avatar_url: None,
            language: "en".into(),
            setup_completed: true,
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("alice");

    let raw = build_subscription_requested_email(
        "bob-feed@example.org",
        "alice-publish@example.org",
        "sub-en",
    );

    engine.ingest_batch(vec![raw]).expect("ingest ok");

    let outbox = store.list_outbox_records().unwrap();
    let resp = outbox
        .iter()
        .find(|r| r.event_type == "subscription_confirmed")
        .expect("SubscriptionConfirmed в outbox");
    let subject = resp
        .subject
        .as_deref()
        .expect("SubscriptionConfirmed должен иметь subject");
    assert!(
        subject.contains("Subscription confirmed"),
        "subject должен быть на английском (язык alice), получили: {subject:?}"
    );
}

#[test]
fn b_accepts_confirmed_and_moves_pending_to_subscriptions() {
    let (store, _tmp) = open();
    let engine = SyncEngine::new(&store).with_profile_id("bob");

    // B имеет pending-подписку
    store
        .save_pending_subscription("bob", "alice-publish@example.org", 1_710_000_000)
        .unwrap();
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "bob".into(),
            nickname: "Боб".into(),
            email_address: "bob-feed@example.org".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();

    let raw = build_subscription_confirmed_email(
        "alice-publish@example.org",
        "bob-feed@example.org",
        "sub-2",
        true,
    );

    let report = engine.ingest_batch(vec![raw]).expect("ingest ok");
    assert!(matches!(
        report.outcomes()[0],
        liveletters_sync::SyncMessageOutcome::Applied { .. }
    ));

    // pending удалён
    assert!(
        store.list_pending_subscriptions("bob").unwrap().is_empty(),
        "pending должна быть удалена"
    );
    // subscriptions содержит
    let subs = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].subscriber_delivery_address, "bob-feed@example.org");
    // local_subscriptions содержит
    assert_eq!(
        store.list_local_subscriptions("bob").unwrap(),
        vec!["alice-publish@example.org".to_string()]
    );
    // display_names содержит профиль A
    assert_eq!(
        store
            .get_display_name("alice-publish@example.org")
            .unwrap()
            .as_deref(),
        Some("Алиса")
    );
}

#[test]
fn b_declines_confirmed_and_removes_pending() {
    let (store, _tmp) = open();
    let engine = SyncEngine::new(&store).with_profile_id("bob");

    store
        .save_pending_subscription("bob", "alice-publish@example.org", 1_710_000_000)
        .unwrap();

    let raw = build_subscription_confirmed_email(
        "alice-publish@example.org",
        "bob-feed@example.org",
        "sub-3",
        false,
    );

    let report = engine.ingest_batch(vec![raw]).expect("ingest ok");
    assert!(matches!(
        report.outcomes()[0],
        liveletters_sync::SyncMessageOutcome::Applied { .. }
    ));

    // pending удалён
    assert!(store.list_pending_subscriptions("bob").unwrap().is_empty());
    // subscriptions пуст
    assert!(
        store
            .list_subscriptions_for_resource("alice-publish@example.org")
            .unwrap()
            .is_empty()
    );
    // local_subscriptions пуст
    assert!(store.list_local_subscriptions("bob").unwrap().is_empty());
}

#[test]
fn b_ignores_confirmed_for_unknown_pending() {
    // Защита от гонки состояний: pending уже отменён, но пришло подтверждение
    let (store, _tmp) = open();
    let engine = SyncEngine::new(&store).with_profile_id("bob");

    let raw = build_subscription_confirmed_email(
        "alice-publish@example.org",
        "bob-feed@example.org",
        "sub-4",
        true,
    );

    let report = engine.ingest_batch(vec![raw]).expect("ingest ok");
    assert!(matches!(
        report.outcomes()[0],
        liveletters_sync::SyncMessageOutcome::Applied { .. }
    ));

    // subscriptions пуст (не должно быть создано ничего)
    assert!(
        store
            .list_subscriptions_for_resource("alice-publish@example.org")
            .unwrap()
            .is_empty()
    );
}
