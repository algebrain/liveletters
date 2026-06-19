//! Тесты команды `lltt comment` через публичный `run`.

mod common;

use liveletters_app_core::{AppCore, CreatePostFromIdentityCommand, Identity, Visibility};
use liveletters_comment::{Args, CommentAction, NewArgs, run};
use liveletters_store::Store;

fn post_id_from(store: &Store) -> String {
    let posts = store.list_posts().expect("list posts");
    assert_eq!(posts.len(), 1);
    posts[0].post_id.clone()
}

fn make_post(home: &common::TestHome, visibility: Visibility) -> String {
    let store = home.open_store();
    store
        .save_identity("alice", "alice@example.test", "alice", None, "ru", true)
        .unwrap();
    store
        .save_author("alice-publish@example.org", "alice", "test")
        .unwrap();
    let core = AppCore::new(&store);
    let ident = Identity {
        publish: "alice-publish@example.org".to_owned(),
    };
    core.create_post_from_identity(CreatePostFromIdentityCommand {
        profile_id: "alice",
        identity: &ident,
        body: "Запись для комментариев",
        visibility,
    })
    .unwrap();
    post_id_from(&store)
}

#[test]
fn comment_new_creates_persisted_comment_with_default_visibility() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");
    let post_id = make_post(&home, Visibility::Public);

    let body_path = home.path().join("c.txt");
    std::fs::write(&body_path, "Первый комментарий").unwrap();

    let args = Args {
        action: CommentAction::New(NewArgs {
            target: post_id.clone(),
            body_file: Some(body_path),
        }),
    };

    run(&ctx, &args).expect("comment new should succeed");

    let store = home.open_store();
    let comments = store.list_comments_for_post(&post_id).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].body, "Первый комментарий");
    assert_eq!(comments[0].visibility, "public");
    assert_eq!(comments[0].author_email, "bob-publish@example.org");
}

#[test]
fn comment_new_inherits_friends_only_visibility_from_post() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");
    let post_id = make_post(&home, Visibility::FriendsOnly);

    let body_path = home.path().join("c.txt");
    std::fs::write(&body_path, "Только для друзей").unwrap();

    let args = Args {
        action: CommentAction::New(NewArgs {
            target: post_id.clone(),
            body_file: Some(body_path),
        }),
    };

    run(&ctx, &args).expect("comment new should succeed");

    let store = home.open_store();
    let comments = store.list_comments_for_post(&post_id).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].visibility, "friends_only");
}

#[test]
fn comment_new_with_parent_creates_reply() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");
    let post_id = make_post(&home, Visibility::Public);

    let root_body = home.path().join("root.txt");
    std::fs::write(&root_body, "Корневой").unwrap();
    let root_args = Args {
        action: CommentAction::New(NewArgs {
            target: post_id.clone(),
            body_file: Some(root_body),
        }),
    };
    run(&ctx, &root_args).expect("root comment should succeed");

    let store = home.open_store();
    let root_id = {
        let comments = store.list_comments_for_post(&post_id).unwrap();
        comments.into_iter().next().unwrap().comment_id
    };

    let reply_body = home.path().join("reply.txt");
    std::fs::write(&reply_body, "Ответ").unwrap();
    let reply_args = Args {
        action: CommentAction::New(NewArgs {
            target: root_id.clone(),
            body_file: Some(reply_body),
        }),
    };
    run(&ctx, &reply_args).expect("reply should succeed");

    let comments = store.list_comments_for_post(&post_id).unwrap();
    assert_eq!(comments.len(), 2);
    let reply = comments
        .iter()
        .find(|c| c.body == "Ответ")
        .expect("reply should be persisted");
    assert_eq!(reply.parent_comment_id.as_deref(), Some(root_id.as_str()));
}

#[test]
fn comment_new_rejects_empty_body() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");
    let post_id = make_post(&home, Visibility::Public);

    let body_path = home.path().join("c.txt");
    std::fs::write(&body_path, "  \n  ").unwrap();

    let args = Args {
        action: CommentAction::New(NewArgs {
            target: post_id,
            body_file: Some(body_path),
        }),
    };

    let err = run(&ctx, &args).unwrap_err();
    assert!(err.to_string().contains("пустое"));
}

#[test]
fn comment_new_to_missing_post_errors() {
    let home = common::TestHome::new();
    home.add_identity("bob");
    let ctx = home.ctx("bob");

    let body_path = home.path().join("c.txt");
    std::fs::write(&body_path, "Текст").unwrap();

    let args = Args {
        action: CommentAction::New(NewArgs {
            target: "post-missing".to_owned(),
            body_file: Some(body_path),
        }),
    };

    let err = run(&ctx, &args).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("не найден") || msg.contains("missing") || msg.contains("записи"),
        "got: {msg}"
    );
}
