use liveletters_store::{Store, SubscriptionRecord};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn record(resource: &str, subscriber: &str, delivery: &str) -> SubscriptionRecord {
    SubscriptionRecord {
        resource_address: resource.into(),
        subscriber_account_id: subscriber.into(),
        subscriber_delivery_address: delivery.into(),
    }
}

#[test]
fn save_subscription_round_trip() {
    let (_dir, store) = open();
    store
        .save_subscription(&record(
            "alice-publish@example.org",
            "acct_bob",
            "bob-feed@example.org",
        ))
        .unwrap();

    let list = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        record(
            "alice-publish@example.org",
            "acct_bob",
            "bob-feed@example.org"
        )
    );
}

#[test]
fn delete_subscription_returns_true_when_row_existed() {
    let (_dir, store) = open();
    store
        .save_subscription(&record(
            "alice-publish@example.org",
            "acct_bob",
            "bob-feed@example.org",
        ))
        .unwrap();

    assert!(
        store
            .delete_subscription("alice-publish@example.org", "acct_bob")
            .unwrap()
    );

    assert!(
        store
            .list_subscriptions_for_resource("alice-publish@example.org")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn list_for_resource_filters_by_resource() {
    let (_dir, store) = open();
    store
        .save_subscription(&record(
            "alice-publish@example.org",
            "acct_bob",
            "bob-feed@example.org",
        ))
        .unwrap();
    store
        .save_subscription(&record(
            "alice-publish@example.org",
            "acct_carol",
            "carol-feed@example.org",
        ))
        .unwrap();
    store
        .save_subscription(&record(
            "dave-publish@example.org",
            "acct_bob",
            "bob-feed@example.org",
        ))
        .unwrap();

    let alice_subs = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(alice_subs.len(), 2);

    let dave_subs = store
        .list_subscriptions_for_resource("dave-publish@example.org")
        .unwrap();
    assert_eq!(dave_subs.len(), 1);
    assert_eq!(dave_subs[0].subscriber_account_id, "acct_bob");
}

#[test]
fn list_for_subscriber_filters_by_subscriber() {
    let (_dir, store) = open();
    store
        .save_subscription(&record(
            "alice-publish@example.org",
            "acct_bob",
            "bob-feed@example.org",
        ))
        .unwrap();
    store
        .save_subscription(&record(
            "dave-publish@example.org",
            "acct_bob",
            "bob-feed2@example.org",
        ))
        .unwrap();
    store
        .save_subscription(&record(
            "alice-publish@example.org",
            "acct_carol",
            "carol-feed@example.org",
        ))
        .unwrap();

    let bob_subs = store.list_subscriptions_for_subscriber("acct_bob").unwrap();
    assert_eq!(bob_subs.len(), 2);
    assert_eq!(bob_subs[0].resource_address, "alice-publish@example.org");
    assert_eq!(bob_subs[1].resource_address, "dave-publish@example.org");

    let carol_subs = store
        .list_subscriptions_for_subscriber("acct_carol")
        .unwrap();
    assert_eq!(carol_subs.len(), 1);
}
