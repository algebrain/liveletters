use liveletters_store::{FriendOfRecord, FriendRecord, PendingFriendRecord, Store};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn save_author(store: &Store, email: &str) {
    store.save_author(email, email, "test").unwrap();
}

#[test]
fn save_friend_is_idempotent_and_lists_by_owner_resource() {
    let (_dir, store) = open();
    for email in ["alice@example.org", "bob@example.org", "eve@example.org"] {
        save_author(&store, email);
    }

    store
        .save_friend("alice@example.org", "bob@example.org")
        .unwrap();
    store
        .save_friend("alice@example.org", "bob@example.org")
        .unwrap();
    store
        .save_friend("eve@example.org", "bob@example.org")
        .unwrap();

    assert_eq!(
        store
            .list_friends_for_resource("alice@example.org")
            .unwrap(),
        vec![FriendRecord {
            owner_resource_email: "alice@example.org".into(),
            friend_email: "bob@example.org".into(),
        }]
    );
    assert!(
        store
            .is_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );
    assert!(
        !store
            .is_friend("bob@example.org", "alice@example.org")
            .unwrap()
    );
}

#[test]
fn pending_friend_updates_last_attempt_without_duplicate() {
    let (_dir, store) = open();
    for email in ["alice@example.org", "bob@example.org"] {
        save_author(&store, email);
    }

    store
        .save_pending_friend(
            "alice",
            "alice@example.org",
            "bob@example.org",
            "bob@example.org",
            10,
        )
        .unwrap();
    store
        .save_pending_friend(
            "alice",
            "alice@example.org",
            "bob@example.org",
            "bob@example.org",
            20,
        )
        .unwrap();

    let pending = store.list_pending_friends("alice").unwrap();
    assert_eq!(
        pending,
        vec![PendingFriendRecord {
            profile_id: "alice".into(),
            owner_resource_email: "alice@example.org".into(),
            friend_email: "bob@example.org".into(),
            subscribed_resource_email: "bob@example.org".into(),
            requested_at: 10,
            last_attempt_at: 20,
        }]
    );
}

#[test]
fn friend_of_is_idempotent_and_lists_by_profile() {
    let (_dir, store) = open();
    save_author(&store, "alice@example.org");

    store.save_friend_of("bob", "alice@example.org").unwrap();
    store.save_friend_of("bob", "alice@example.org").unwrap();

    assert_eq!(
        store.list_friend_of("bob").unwrap(),
        vec![FriendOfRecord {
            profile_id: "bob".into(),
            resource_email: "alice@example.org".into(),
        }]
    );
}

#[test]
fn deleting_absent_friend_records_is_ok() {
    let (_dir, store) = open();

    assert!(
        !store
            .delete_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );
    store
        .remove_pending_friend("alice", "alice@example.org", "bob@example.org")
        .unwrap();
}
