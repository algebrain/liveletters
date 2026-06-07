use liveletters_app_core::{
    AppCore, ListSubscriptionsQuery, SubscribeCommand, SubscribeResult, UnsubscribeCommand,
};
use liveletters_protocol::decode_message;
use liveletters_store::{OutboxDelivery, Store, SubscriptionRecord};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn subscribe_writes_outbox_with_direct_delivery_and_does_not_touch_subscriptions_table() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);

    let result: SubscribeResult = core
        .subscribe(SubscribeCommand {
            profile_id: "default",
            resource_address: "alice-publish@example.org",
            subscriber_delivery_address: "bob-feed@example.org",
            created_at: 1_700_000_000,
        })
        .unwrap();

    assert_eq!(result.resource_address, "alice-publish@example.org");
    assert_eq!(result.delivery_address, "bob-feed@example.org");

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].resource_id, "alice-publish@example.org");
    assert_eq!(
        outbox[0].delivery,
        OutboxDelivery::Direct(vec!["alice-publish@example.org".to_owned()])
    );
    let decoded = decode_message(&outbox[0].message_body).expect("message should decode");
    assert_eq!(decoded.envelope().event_type(), "subscription_changed");

    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert!(
        records.is_empty(),
        "локальная подписка не должна записываться в таблицу подписчиков"
    );
}

#[test]
fn unsubscribe_writes_outbox_with_direct_delivery_and_does_not_touch_subscriptions_table() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);

    core.subscribe(SubscribeCommand {
        profile_id: "default",
        resource_address: "alice-publish@example.org",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 1,
    })
    .unwrap();

    let result = core
        .unsubscribe(UnsubscribeCommand {
            profile_id: "default",
            resource_address: "alice-publish@example.org",
            subscriber_delivery_address: "bob-feed@example.org",
            created_at: 2,
        })
        .unwrap();

    assert_eq!(result.resource_address, "alice-publish@example.org");

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 2);
    assert!(outbox.iter().all(|r| {
        let decoded = decode_message(&r.message_body).expect("message should decode");
        decoded.envelope().event_type() == "subscription_changed"
    }));
    assert!(outbox.iter().all(|r| matches!(
        r.delivery,
        OutboxDelivery::Direct(ref addrs) if addrs == &vec!["alice-publish@example.org".to_owned()]
    )));

    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert!(
        records.is_empty(),
        "локальная отписка не должна удалять строку чужой подписки"
    );
}

#[test]
fn list_subscriptions_returns_owned_from_table_and_subscribed_from_query() {
    let (_dir, store) = open();

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "bob-feed@example.org".into(),
        })
        .unwrap();
    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "carol-feed@example.org".into(),
        })
        .unwrap();

    let core = AppCore::new(&store);
    let list = core
        .list_subscriptions(ListSubscriptionsQuery {
            owned_resource_address: "alice-publish@example.org",
            subscribed_addresses: &[
                "dave-publish@example.org".to_string(),
                "eve-publish@example.org".to_string(),
            ],
        })
        .unwrap();

    assert_eq!(list.owned_subscribers().len(), 2);
    assert_eq!(
        list.subscribed_addresses(),
        &["dave-publish@example.org", "eve-publish@example.org"]
    );
}

#[test]
fn subscribe_rejects_invalid_address() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);

    let err = core
        .subscribe(SubscribeCommand {
            profile_id: "default",
            resource_address: "not-an-address",
            subscriber_delivery_address: "bob-feed@example.org",
            created_at: 1,
        })
        .unwrap_err();

    assert!(matches!(err, liveletters_app_core::AppCoreError::Domain(_)));
}
