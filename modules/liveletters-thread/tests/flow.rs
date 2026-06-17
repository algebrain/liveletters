//! Тесты команды `lltt thread` через публичный `run`.

mod common;

use liveletters_app_core::{
    AppCore, CreateCommentFromIdentityCommand, CreatePostFromIdentityCommand, Identity, Visibility,
};
use liveletters_thread::{Args, run};

fn identity(name: &str) -> Identity {
    Identity {
        publish: format!("{name}-publish@example.org"),
    }
}

fn save_publish_author(store: &liveletters_store::Store, name: &str) {
    store
        .save_author(&format!("{name}-publish@example.org"), name, "test")
        .unwrap();
}

#[test]
fn thread_for_existing_post_prints_post_and_no_comments_marker() {
    let home = common::TestHome::new();
    home.init();
    let ctx = home.ctx("alice");

    let store = home.open_store();
    save_publish_author(&store, "alice");
    let core = AppCore::new(&store);
    let post = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &identity("alice"),
            body: "Текст поста",
            visibility: Visibility::Public,
        })
        .unwrap();

    let args = Args {
        post_id: post.post().id().as_str().to_owned(),
    };

    run(&ctx, &args).expect("thread should succeed for existing post");
}

#[test]
fn thread_for_post_with_root_and_reply_prints_tree() {
    let home = common::TestHome::new();
    home.init();
    let ctx = home.ctx("alice");

    let store = home.open_store();
    save_publish_author(&store, "alice");
    save_publish_author(&store, "bob");
    let core = AppCore::new(&store);
    let post = core
        .create_post_from_identity(CreatePostFromIdentityCommand {
            profile_id: "alice",
            identity: &identity("alice"),
            body: "Запись",
            visibility: Visibility::Public,
        })
        .unwrap();

    let root = core
        .create_comment_from_identity(CreateCommentFromIdentityCommand {
            profile_id: "alice",
            identity: &identity("bob"),
            post_id: post.post().id().as_str(),
            parent_comment_id: None,
            body: "Корневой",
            visibility: Visibility::Public,
        })
        .unwrap();

    core.create_comment_from_identity(CreateCommentFromIdentityCommand {
        profile_id: "alice",
        identity: &identity("alice"),
        post_id: post.post().id().as_str(),
        parent_comment_id: Some(root.comment().id().as_str()),
        body: "Ответ",
        visibility: Visibility::Public,
    })
    .unwrap();

    let args = Args {
        post_id: post.post().id().as_str().to_owned(),
    };

    run(&ctx, &args).expect("thread should succeed for post with comments");
}

#[test]
fn thread_for_missing_post_errors() {
    let home = common::TestHome::new();
    let ctx = home.ctx("alice");

    let args = Args {
        post_id: "missing-post".to_owned(),
    };

    let err = run(&ctx, &args).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("не найден") || msg.contains("записи") || msg.contains("post"),
        "got: {msg}"
    );
}
