//! Тесты команды `create_comment` с фокусом на видимость (`public` / `friends_only`).

use liveletters_app_core::{AppCore, CreateCommentCommand, CreatePostCommand, Visibility};
use liveletters_store::{Store, UserSettingsRecord};
use serde_json::Value;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn save_user(store: &Store) {
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "default".into(),
            nickname: "alice".into(),
            email_address: "alice@example.test".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
}

fn setup_post(store: &Store) {
    let core = AppCore::new(store);
    core.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "acct_alice",
        created_at: 1_700_000_000,
        body: "Тело",
        visibility: Visibility::Public,
    })
    .unwrap();
}

fn create_with(store: &Store, visibility: Visibility) {
    let core = AppCore::new(store);
    core.create_comment(CreateCommentCommand {
        profile_id: "default",
        comment_id: "comment-1",
        post_id: "post-1",
        parent_comment_id: None,
        author_id: "acct_bob",
        created_at: 1_700_000_100,
        body: "Комментарий",
        visibility,
    })
    .unwrap();
}

#[test]
fn create_comment_with_friends_only_persists_visibility() {
    let (_dir, store) = open();
    save_user(&store);
    setup_post(&store);
    create_with(&store, Visibility::FriendsOnly);

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
fn create_comment_with_public_persists_visibility() {
    let (_dir, store) = open();
    save_user(&store);
    setup_post(&store);
    create_with(&store, Visibility::Public);

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
