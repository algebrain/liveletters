mod common;

use common::open_temp_store;
use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};
use liveletters_store::SubscriptionRecord;
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

fn subscription_email(
    event_id: &str,
    resource_address: &str,
    subscriber_delivery_address: &str,
    active: bool,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "subscription_changed", resource_address, event_id).unwrap(),
        "Подписка",
        DomainEventPayload::SubscriptionChanged {
            resource_address: resource_address.into(),
            subscriber_delivery_address: subscriber_delivery_address.into(),
            active,
            created_at: 1_710_000_000,
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "bob@example.test",
        resource_address,
        "Sync fixture",
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
        &message,
    )
    .unwrap();

    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

#[test]
fn apply_subscription_changed_persists_record() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![subscription_email(
            "sub-1",
            "alice-publish@example.org",
            "bob-feed@example.org",
            true,
        )])
        .unwrap();

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));

    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(
        records,
        vec![SubscriptionRecord {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
        }]
    );
}

#[test]
fn apply_unsubscription_changed_removes_record() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    engine
        .ingest_batch(vec![subscription_email(
            "sub-1",
            "alice-publish@example.org",
            "bob-feed@example.org",
            true,
        )])
        .unwrap();

    engine
        .ingest_batch(vec![subscription_email(
            "unsub-1",
            "alice-publish@example.org",
            "bob-feed@example.org",
            false,
        )])
        .unwrap();

    assert!(
        store
            .list_subscriptions_for_resource("alice-publish@example.org")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn apply_legacy_subscription_changed_ignores_account_id() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);
    let raw_message = r#"{
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
            "subscriber_account_id": "not-a-real-global-id",
            "subscriber_delivery_address": "bob-feed@example.org",
            "action": "subscribe",
            "created_at": 1710000000
        }
    }"#;
    let outgoing = build_protocol_email(
        "bob@example.test",
        "alice-publish@example.org",
        "Sync fixture",
        &liveletters_protocol::decode_message(raw_message).unwrap(),
    )
    .unwrap();

    engine
        .ingest_batch(vec![ReceivedEmail {
            message_id: "message-sub-legacy".into(),
            raw_message: outgoing.raw_message,
        }])
        .unwrap();

    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(
        records,
        vec![SubscriptionRecord {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
        }]
    );
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
