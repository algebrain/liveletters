//! Тесты команды `create_post` с фокусом на видимость (`public` / `friends_only`).

use liveletters_app_core::{AppCore, CreatePostCommand, CreatePostResult, Visibility};
use liveletters_store::Store;
use serde_json::Value;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    save_user(&store);
    (dir, store)
}

fn save_user(store: &Store) {
    store
        .save_identity("default", "alice@example.test", "alice", None, "ru", true)
        .unwrap();
}

fn create_with(store: &Store, visibility: Visibility) -> CreatePostResult {
    let core = AppCore::new(store);
    core.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "alice@example.test",
        author_id: "alice@example.test",
        created_at: 1_700_000_000,
        body: "Привет, мир",
        visibility,
    })
    .unwrap()
}

#[test]
fn create_post_with_friends_only_persists_visibility() {
    let (_dir, store) = open();

    let result = create_with(&store, Visibility::FriendsOnly);

    let post = store.get_post_record("post-1").unwrap().unwrap();
    assert_eq!(post.visibility, "friends_only");
    assert_eq!(result.post().visibility(), Visibility::FriendsOnly);

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    let envelope: Value = serde_json::from_str(&outbox[0].message_body).unwrap();
    assert_eq!(envelope["envelope"]["event_type"], "post_created");
    assert_eq!(envelope["payload"]["visibility"], "friends_only");
    assert_eq!(envelope["payload"]["body"], "Привет, мир");
    assert_eq!(envelope["payload"]["body_format"], "plain");
    // human_readable_body намеренно не сериализуется в JSON
    // (см. message.rs: skip_serializing default). Локализованный текст
    // хранится в отдельной колонке outbox.
    assert!(envelope.get("human_readable_body").is_none());
    let body = outbox[0]
        .human_readable_body
        .as_deref()
        .expect("тело должно быть в отдельной колонке outbox");
    assert_eq!(
        body,
        "Новая запись в журнале alice@example.test:\n\nПривет, мир\n\n— LiveLetters"
    );
}

#[test]
fn create_post_with_public_persists_visibility() {
    let (_dir, store) = open();

    let result = create_with(&store, Visibility::Public);

    let post = store.get_post_record("post-1").unwrap().unwrap();
    assert_eq!(post.visibility, "public");
    assert_eq!(result.post().visibility(), Visibility::Public);

    let outbox = store.list_outbox_records().unwrap();
    let envelope: Value = serde_json::from_str(&outbox[0].message_body).unwrap();
    assert_eq!(envelope["payload"]["visibility"], "public");
}

#[test]
fn create_post_fails_when_nickname_and_email_are_both_empty() {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    let core = AppCore::new(&store);
    let result = core.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "alice@example.test",
        author_id: "alice@example.test",
        created_at: 1_700_000_000,
        body: "Привет, мир",
        visibility: Visibility::Public,
    });
    assert!(
        result.is_err(),
        "RED: create_post should fail when UserSettingsRecord is missing or has empty nickname+email"
    );
}

#[test]
fn create_post_uses_email_when_nickname_is_empty() {
    let (_dir, store) = open();
    store
        .save_identity("default", "alice@example.test", "", None, "ru", true)
        .unwrap();
    let core = AppCore::new(&store);
    core.create_post(CreatePostCommand {
        profile_id: "default",
        post_id: "post-1",
        resource_id: "alice@example.test",
        author_id: "alice@example.test",
        created_at: 1_700_000_000,
        body: "Привет, мир",
        visibility: Visibility::Public,
    })
    .unwrap();
    let outbox = store.list_outbox_records().unwrap();
    let envelope: Value = serde_json::from_str(&outbox[0].message_body).unwrap();
    assert_eq!(
        envelope["payload"]["actor_id"], "alice@example.test",
        "RED: actor_id should fall back to email when nickname is empty"
    );
}
