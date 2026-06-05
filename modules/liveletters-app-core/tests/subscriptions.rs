use liveletters_app_core::{
    AppCore, ListSubscriptionsQuery, SubscribeCommand, SubscribeResult, UnsubscribeCommand,
};
use liveletters_store::Store;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

#[test]
fn subscribe_writes_outbox_record_and_persists_subscription() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);

    let result: SubscribeResult = core
        .subscribe(SubscribeCommand {
            resource_address: "alice-publish@example.org",
            subscriber_account_id: "acct_bob",
            subscriber_delivery_address: "bob-feed@example.org",
            created_at: 1_700_000_000,
        })
        .unwrap();

    assert_eq!(result.resource_address, "alice-publish@example.org");
    assert_eq!(result.delivery_address, "bob-feed@example.org");

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_type, "subscription_changed");
    assert_eq!(outbox[0].resource_id, "alice-publish@example.org");

    let records = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].subscriber_account_id, "acct_bob");
}

#[test]
fn unsubscribe_removes_subscription_and_writes_outbox() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);

    core.subscribe(SubscribeCommand {
        resource_address: "alice-publish@example.org",
        subscriber_account_id: "acct_bob",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 1,
    })
    .unwrap();

    let result = core
        .unsubscribe(UnsubscribeCommand {
            resource_address: "alice-publish@example.org",
            subscriber_account_id: "acct_bob",
            created_at: 2,
        })
        .unwrap();

    assert_eq!(result.resource_address, "alice-publish@example.org");

    assert!(
        store
            .list_subscriptions_for_resource("alice-publish@example.org")
            .unwrap()
            .is_empty()
    );

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 2);
    assert!(
        outbox
            .iter()
            .all(|r| r.event_type == "subscription_changed")
    );
}

#[test]
fn list_subscriptions_returns_owned_and_subscribed() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);

    core.subscribe(SubscribeCommand {
        resource_address: "alice-publish@example.org",
        subscriber_account_id: "acct_bob",
        subscriber_delivery_address: "bob-feed@example.org",
        created_at: 1,
    })
    .unwrap();
    core.subscribe(SubscribeCommand {
        resource_address: "alice-publish@example.org",
        subscriber_account_id: "acct_carol",
        subscriber_delivery_address: "carol-feed@example.org",
        created_at: 2,
    })
    .unwrap();

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
            resource_address: "not-an-address",
            subscriber_account_id: "acct_bob",
            subscriber_delivery_address: "bob-feed@example.org",
            created_at: 1,
        })
        .unwrap_err();

    assert!(matches!(err, liveletters_app_core::AppCoreError::Domain(_)));
}
