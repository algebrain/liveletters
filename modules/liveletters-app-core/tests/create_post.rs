//! Тесты команды `create_post` с фокусом на видимость (`public` / `friends_only`).

use liveletters_app_core::{AppCore, CreatePostCommand, CreatePostResult, Visibility};
use liveletters_store::Store;
use serde_json::Value;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    (dir, store)
}

fn create_with(store: &Store, visibility: Visibility) -> CreatePostResult {
    let core = AppCore::new(store);
    core.create_post(CreatePostCommand {
        post_id: "post-1",
        resource_id: "blog-1",
        author_id: "acct_alice",
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
