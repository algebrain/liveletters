//! Тесты `create_comment_from_identity`: автоподстановка `comment_id`, `author_id`, `created_at`.

use liveletters_app_core::{
    AppCore, CreateCommentFromIdentityCommand, CreatePostFromIdentityCommand, Identity, Visibility,
};
use liveletters_store::{Store, UserSettingsRecord};
use tempfile::tempdir;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempdir().unwrap();
    let store = Store::open_for_home_dir(dir.path()).unwrap();
    save_user(&store);
    (dir, store)
}

fn save_user(store: &Store) {
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "alice".into(),
            nickname: "alice".into(),
            email_address: "alice@example.test".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
}

fn identity() -> Identity {
    Identity {
        publish: "alice-publish@example.org".to_owned(),
    }
}

#[test]
fn create_comment_from_identity_derives_fields_and_persists_comment() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);
    let ident = identity();

    let post = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            body: "Запись",
            visibility: Visibility::Public,
        })
        .unwrap();

    let result = core
        .create_comment_from_identity(CreateCommentFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            post_id: post.post().id().as_str(),
            parent_comment_id: None,
            body: "Комментарий",
            visibility: Visibility::FriendsOnly,
        })
        .unwrap();

    let comment = result.comment();
    assert!(comment.id().as_str().starts_with("comment-"));
    assert_eq!(comment.author_id().as_str(), "alice-publish@example.org");
    assert_eq!(comment.visibility(), Visibility::FriendsOnly);
    assert_eq!(comment.body().as_str(), "Комментарий");

    let record = store
        .get_comment_record(comment.id().as_str())
        .unwrap()
        .expect("comment must be persisted");
    assert_eq!(record.author_id, "alice-publish@example.org");
    assert_eq!(record.visibility, "friends_only");
    assert_eq!(record.post_id, post.post().id().as_str());
}

#[test]
fn create_comment_from_identity_uses_parent_when_provided() {
    let (_dir, store) = open();
    let core = AppCore::new(&store);
    let ident = identity();

    let post = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            body: "Запись",
            visibility: Visibility::Public,
        })
        .unwrap();

    let parent = core
        .create_comment_from_identity(CreateCommentFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            post_id: post.post().id().as_str(),
            parent_comment_id: None,
            body: "Корневой",
            visibility: Visibility::Public,
        })
        .unwrap();

    let child = core
        .create_comment_from_identity(CreateCommentFromIdentityCommand {
            profile_id: "alice",
            identity: &ident,
            post_id: post.post().id().as_str(),
            parent_comment_id: Some(parent.comment().id().as_str()),
            body: "Ответ",
            visibility: Visibility::Public,
        })
        .unwrap();

    assert_eq!(
        child.comment().parent_comment_id().unwrap().as_str(),
        parent.comment().id().as_str()
    );
}
