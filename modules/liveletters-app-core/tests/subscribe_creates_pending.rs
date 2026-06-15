//! `subscribe` отправляет `SubscriptionRequested` в outbox и кладёт
//! запись в `pending_subscriptions`. Не создаёт запись в `subscriptions`
//! и `local_subscriptions` — это будет сделано в этапе 5 после получения
//! `SubscriptionConfirmed`.

use liveletters_app_core::{AppCore, SubscribeCommand};
use liveletters_protocol::decode_message;
use liveletters_store::OutboxDelivery;
use liveletters_store::UserSettingsRecord;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, liveletters_store::Store) {
    let dir = tempdir().unwrap();
    let store = liveletters_store::Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn save_user(store: &liveletters_store::Store) {
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "default".into(),
            nickname: "default".into(),
            email_address: "alice@example.org".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
}

#[test]
fn subscribe_creates_outbox_pending_does_not_create_subscription() {
    let (_dir, store) = open();
    save_user(&store);
    let core = AppCore::new(&store);

    core.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "bob@example.org",
        subscriber_delivery_address: "alice@example.org",
        created_at: 1_700_000_000,
    })
    .unwrap();

    // outbox получил SubscriptionRequested
    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1, "должна быть одна запись в outbox");
    assert_eq!(outbox[0].event_type, "subscription_requested");
    assert!(
        outbox[0].message_id.is_some(),
        "Message-ID должен быть заполнен"
    );
    let decoded = decode_message(&outbox[0].message_body).expect("decode");
    assert!(matches!(
        decoded.envelope().event_type(),
        "subscription_requested"
    ));

    // pending_subscriptions содержит запись
    let pending = store.list_pending_subscriptions("default").unwrap();
    assert_eq!(pending.len(), 1, "должна быть одна pending-подписка");
    assert_eq!(pending[0].resource_address, "bob@example.org");
    assert_eq!(pending[0].requested_at, 1_700_000_000);

    // subscriptions пуст
    assert!(
        store
            .list_subscriptions_for_resource("bob@example.org")
            .unwrap()
            .is_empty(),
        "SubscriptionRequested не должен сразу создавать подтверждённую подписку"
    );

    // local_subscriptions пуст
    assert!(
        store
            .list_local_subscriptions("default")
            .unwrap()
            .is_empty(),
        "локальная подписка появится только после SubscriptionConfirmed"
    );
}

#[test]
fn repeated_subscribe_updates_last_attempt_at_only() {
    let (_dir, store) = open();
    save_user(&store);
    let core = AppCore::new(&store);

    core.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "bob@example.org",
        subscriber_delivery_address: "alice@example.org",
        created_at: 1_700_000_000,
    })
    .unwrap();
    core.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "bob@example.org",
        subscriber_delivery_address: "alice@example.org",
        created_at: 1_700_000_500,
    })
    .unwrap();

    let pending = store.list_pending_subscriptions("default").unwrap();
    assert_eq!(pending.len(), 1, "не должно быть дубликатов");
    assert_eq!(pending[0].requested_at, 1_700_000_000);
    assert_eq!(pending[0].last_attempt_at, 1_700_000_500);

    // outbox получил два письма (по одному на каждую попытку)
    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 2);
    for r in &outbox {
        assert_eq!(r.event_type, "subscription_requested");
        assert_eq!(
            r.delivery,
            OutboxDelivery::Direct(vec!["bob@example.org".to_owned()])
        );
    }
}
