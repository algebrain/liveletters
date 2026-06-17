//! Тесты `create_post_from_identity`: автоподстановка `post_id`, `resource_email`, `author_email`, `created_at`.

use liveletters_app_core::{AppCore, CreatePostFromIdentityCommand, Identity, Visibility};
use liveletters_store::Store;
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    save_user(&store);
    (dir, store)
}

fn save_user(store: &Store) {
    store
        .save_identity("alice", "alice@example.test", "alice", None, "ru", true)
        .unwrap();
    store
        .save_author("alice-publish@example.org", "alice", "test")
        .unwrap();
}

fn identity() -> Identity {
    Identity {
        publish: "alice-publish@example.org".to_owned(),
    }
}

#[test]
fn create_post_from_identity_derives_fields_and_persists_post() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);
    let ident = identity();

    let result = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            body: "Привет из identity",
            visibility: Visibility::FriendsOnly,
        })
        .unwrap();

    let post = result.post();
    assert!(post.id().as_str().starts_with("post-"));
    assert_eq!(post.resource_id().as_str(), "alice-publish@example.org");
    assert_eq!(post.author_id().as_str(), "alice-publish@example.org");
    assert_eq!(post.visibility(), Visibility::FriendsOnly);

    let record = store
        .get_post_record(post.id().as_str())
        .unwrap()
        .expect("post must be persisted");
    assert_eq!(record.resource_email, "alice-publish@example.org");
    assert_eq!(record.author_email, "alice-publish@example.org");
    assert_eq!(record.visibility, "friends_only");
}

#[test]
fn create_post_from_identity_generates_unique_ids_across_calls() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);
    let ident = identity();

    let first = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            body: "Первый",
            visibility: Visibility::Public,
        })
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let second = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            body: "Второй",
            visibility: Visibility::Public,
        })
        .unwrap();

    assert_ne!(first.post().id().as_str(), second.post().id().as_str());
    assert!(first.post().id().as_str() < second.post().id().as_str());

    let all = store.list_posts().unwrap();
    assert_eq!(all.len(), 2);
}
