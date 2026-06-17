//! Проверка CRUD для `pending_subscriptions` и `authors`.

mod common;

use common::open_temp_store;

#[test]
fn pending_subscription_round_trip() {
    let (store, _tmp) = open_temp_store();
    common::ensure_author(&store, "bob@example.org", "bob");

    store
        .save_pending_subscription("alice", "bob@example.org", 1_700_000_000)
        .unwrap();
    store
        .update_pending_last_attempt("alice", "bob@example.org", 1_700_000_100)
        .unwrap();

    let list = store.list_pending_subscriptions("alice").unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].profile_id, "alice");
    assert_eq!(list[0].resource_email, "bob@example.org");
    assert_eq!(list[0].requested_at, 1_700_000_000);
    assert_eq!(list[0].last_attempt_at, 1_700_000_100);

    let found = store
        .find_pending_subscription("alice", "bob@example.org")
        .unwrap();
    assert!(found.is_some());

    store
        .remove_pending_subscription("alice", "bob@example.org")
        .unwrap();
    assert!(
        store
            .list_pending_subscriptions("alice")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn repeated_save_pending_does_not_duplicate() {
    let (store, _tmp) = open_temp_store();
    common::ensure_author(&store, "bob@example.org", "bob");

    store
        .save_pending_subscription("alice", "bob@example.org", 1_700_000_000)
        .unwrap();
    store
        .save_pending_subscription("alice", "bob@example.org", 1_700_000_500)
        .unwrap();

    let list = store.list_pending_subscriptions("alice").unwrap();
    assert_eq!(list.len(), 1, "не должно быть дубликатов");
    assert_eq!(list[0].requested_at, 1_700_000_000);
    assert_eq!(list[0].last_attempt_at, 1_700_000_500);
}

#[test]
fn author_round_trip_and_overwrite() {
    let (store, _tmp) = open_temp_store();

    store
        .save_author("alice@example.org", "Алиса", "subscription_confirmed")
        .unwrap();
    let author = store.get_author("alice@example.org").unwrap();
    assert_eq!(author.as_ref().map(|a| a.nickname.as_str()), Some("Алиса"));

    store
        .save_author("alice@example.org", "Алина", "post_created")
        .unwrap();
    let author = store.get_author("alice@example.org").unwrap();
    assert_eq!(
        author.as_ref().map(|a| a.nickname.as_str()),
        Some("Алина"),
        "новый источник должен перезаписывать имя"
    );

    let list = store.list_authors().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].source, "post_created");
}
