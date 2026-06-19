//! Тесты команды `create_comment`: комментарий наследует видимость исходной записи.

use liveletters_app_core::{AppCore, CreateCommentCommand, CreatePostCommand, Visibility};
use liveletters_store::Store;
use serde_json::Value;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn save_user(store: &Store) {
    store
        .save_identity("default", "alice@example.test", "alice", None, "ru", true)
        .unwrap();
}

fn setup_post(store: &Store, visibility: Visibility) {
    let core = AppCore::new(store);
    core.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "alice@example.test",
        author_id: "alice@example.test",
        created_at: 1_700_000_000,
        body: "Тело",
        visibility,
    })
    .unwrap();
}

fn create_with(store: &Store) {
    store
        .save_author("bob@example.org", "bob", "test")
        .expect("save comment author");
    let core = AppCore::new(store);
    core.create_comment(CreateCommentCommand {
        profile_id: "default",
        comment_id: "comment-1",
        post_id: "post-1",
        parent_comment_id: None,
        author_id: "bob@example.org",
        created_at: 1_700_000_100,
        body: "Комментарий",
    })
    .unwrap();
}

#[test]
fn create_comment_inherits_friends_only_from_post() {
    let (_dir, store) = open();
    save_user(&store);
    setup_post(&store, Visibility::FriendsOnly);
    create_with(&store);

    let record = store.get_comment_record("comment-1").unwrap().unwrap();
    assert_eq!(record.visibility, "friends_only");

    let outbox = store.list_outbox_records().unwrap();
    let comment_event = outbox
        .iter()
        .find(|r| r.event_id == "comment-created:comment-1")
        .expect("comment_created outbox row should exist");
    let envelope: Value = serde_json::from_str(&comment_event.message_body).unwrap();
    assert_eq!(envelope["envelope"]["event_type"], "comment_created");
    assert_eq!(envelope["payload"]["visibility"], "friends_only");
}

#[test]
fn create_comment_inherits_public_from_post() {
    let (_dir, store) = open();
    save_user(&store);
    setup_post(&store, Visibility::Public);
    create_with(&store);

    let record = store.get_comment_record("comment-1").unwrap().unwrap();
    assert_eq!(record.visibility, "public");

    let outbox = store.list_outbox_records().unwrap();
    let comment_event = outbox
        .iter()
        .find(|r| r.event_id == "comment-created:comment-1")
        .expect("comment_created outbox row should exist");
    let envelope: Value = serde_json::from_str(&comment_event.message_body).unwrap();
    assert_eq!(envelope["payload"]["visibility"], "public");
}
