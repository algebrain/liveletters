use liveletters_app_core::{
    AppCore, CreatePostCommand, FriendCommand, SubscribeCommand, Visibility,
};
use liveletters_protocol::{DomainEventPayload, decode_message};
use liveletters_store::{OutboxDelivery, Store};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn save_identity(store: &Store, profile: &str, email: &str, nickname: &str) {
    store
        .save_identity(profile, email, nickname, None, "ru", true)
        .unwrap();
}

fn save_author(store: &Store, email: &str) {
    store.save_author(email, email, "test").unwrap();
}

#[test]
fn friend_existing_local_subscription_saves_friend_and_sends_friend_added() {
    let (_dir, store) = open();
    save_identity(&store, "alice", "alice@example.org", "Алиса");
    save_author(&store, "bob@example.org");
    store
        .add_local_subscription("alice", "bob@example.org")
        .unwrap();

    let core = AppCore::new(&store);
    core.friend(FriendCommand {
        profile_id: "alice",
        owner_resource_address: "alice@example.org",
        friend_address: "bob@example.org",
        created_at: 1_770_000_000,
    })
    .unwrap();

    assert!(
        store
            .is_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );
    assert!(store.list_pending_friends("alice").unwrap().is_empty());

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_type, "friend_added");
    assert_eq!(
        outbox[0].delivery,
        OutboxDelivery::Direct(vec!["bob@example.org".into()])
    );
    assert!(
        outbox[0]
            .human_readable_body
            .as_deref()
            .unwrap_or("")
            .contains("друз")
    );
    let decoded = decode_message(&outbox[0].message_body).unwrap();
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::FriendAdded {
            resource_id,
            friend_address,
            ..
        } if resource_id == "alice@example.org" && friend_address == "bob@example.org"
    ));
}

#[test]
fn friend_without_local_subscription_uses_plain_subscription_request() {
    let (_dir, store) = open();
    save_identity(&store, "alice", "alice@example.org", "Алиса");

    let core = AppCore::new(&store);
    core.friend(FriendCommand {
        profile_id: "alice",
        owner_resource_address: "alice@example.org",
        friend_address: "bob@example.org",
        created_at: 1_770_000_000,
    })
    .unwrap();

    assert!(
        !store
            .is_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );
    assert_eq!(store.list_pending_friends("alice").unwrap().len(), 1);

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_type, "subscription_requested");
    let decoded = decode_message(&outbox[0].message_body).unwrap();
    let encoded: serde_json::Value = serde_json::from_str(&outbox[0].message_body).unwrap();
    assert!(encoded["payload"].get("purpose").is_none());
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::SubscriptionRequested {
            resource_id,
            subscriber_delivery_address,
            ..
        } if resource_id == "bob@example.org"
            && subscriber_delivery_address == "alice@example.org"
    ));
}

#[test]
fn confirmed_pending_friend_saves_friend_and_enqueues_friend_added() {
    let (_dir, store) = open();
    save_identity(&store, "alice", "alice@example.org", "Алиса");
    save_author(&store, "bob@example.org");
    store
        .save_pending_friend(
            "alice",
            "alice@example.org",
            "bob@example.org",
            "bob@example.org",
            10,
        )
        .unwrap();

    let core = AppCore::new(&store);
    core.complete_pending_friend_after_subscription("alice", "bob@example.org")
        .unwrap();

    assert!(
        store
            .is_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );
    assert!(store.list_pending_friends("alice").unwrap().is_empty());
    assert_eq!(
        store.list_outbox_records().unwrap()[0].event_type,
        "friend_added"
    );
}

#[test]
fn friends_only_post_uses_friends_audience_delivery() {
    let (_dir, store) = open();
    save_identity(&store, "alice", "alice@example.org", "Алиса");
    let core = AppCore::new(&store);

    core.create_post(CreatePostCommand {
        profile_id: "alice",
        post_id: "post-1",
        resource_id: "alice@example.org",
        author_id: "alice@example.org",
        created_at: 1,
        body: "Приватная запись",
        visibility: Visibility::FriendsOnly,
    })
    .unwrap();

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(
        outbox[0].delivery,
        OutboxDelivery::ResourceFriends {
            visibility: "friends_only".into()
        }
    );
}

#[test]
fn subscribe_still_creates_plain_subscription_request() {
    let (_dir, store) = open();
    save_identity(&store, "alice", "alice@example.org", "Алиса");
    let core = AppCore::new(&store);

    core.subscribe(SubscribeCommand {
        profile_id: "alice",
        resource_address: "bob@example.org",
        subscriber_delivery_address: "alice@example.org",
        created_at: 1,
    })
    .unwrap();

    let outbox = store.list_outbox_records().unwrap();
    let json: serde_json::Value = serde_json::from_str(&outbox[0].message_body).unwrap();
    assert!(json["payload"].get("purpose").is_none());
}
