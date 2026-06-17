use liveletters_store::{Store, SubscriptionRecord};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn record(resource: &str, delivery: &str) -> SubscriptionRecord {
    SubscriptionRecord {
        resource_email: resource.into(),
        subscriber_email: delivery.into(),
    }
}

fn save_subscription(store: &Store, resource: &str, delivery: &str) {
    store.save_author(resource, resource, "test").unwrap();
    store.save_author(delivery, delivery, "test").unwrap();
    store
        .save_subscription(&record(resource, delivery))
        .unwrap();
}

#[test]
fn save_subscription_round_trip() {
    let (_dir, store) = open();
    save_subscription(&store, "alice-publish@example.org", "bob-feed@example.org");

    let list = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        record("alice-publish@example.org", "bob-feed@example.org")
    );
}

#[test]
fn delete_subscription_returns_true_when_row_existed() {
    let (_dir, store) = open();
    save_subscription(&store, "alice-publish@example.org", "bob-feed@example.org");

    assert!(
        store
            .delete_subscription("alice-publish@example.org", "bob-feed@example.org")
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
    save_subscription(&store, "alice-publish@example.org", "bob-feed@example.org");
    save_subscription(
        &store,
        "alice-publish@example.org",
        "carol-feed@example.org",
    );
    save_subscription(&store, "dave-publish@example.org", "bob-feed@example.org");

    let alice_subs = store
        .list_subscriptions_for_resource("alice-publish@example.org")
        .unwrap();
    assert_eq!(alice_subs.len(), 2);

    let dave_subs = store
        .list_subscriptions_for_resource("dave-publish@example.org")
        .unwrap();
    assert_eq!(dave_subs.len(), 1);
    assert_eq!(dave_subs[0].subscriber_email, "bob-feed@example.org");
}

#[test]
fn list_for_subscriber_filters_by_subscriber() {
    let (_dir, store) = open();
    save_subscription(&store, "alice-publish@example.org", "bob-feed@example.org");
    save_subscription(&store, "dave-publish@example.org", "bob-feed@example.org");
    save_subscription(
        &store,
        "alice-publish@example.org",
        "carol-feed@example.org",
    );

    let bob_subs = store
        .list_subscriptions_for_subscriber("bob-feed@example.org")
        .unwrap();
    assert_eq!(bob_subs.len(), 2);
    assert_eq!(bob_subs[0].resource_email, "alice-publish@example.org");
    assert_eq!(bob_subs[1].resource_email, "dave-publish@example.org");

    let carol_subs = store
        .list_subscriptions_for_subscriber("carol-feed@example.org")
        .unwrap();
    assert_eq!(carol_subs.len(), 1);
}
