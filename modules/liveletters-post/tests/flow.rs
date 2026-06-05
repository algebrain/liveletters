//! Тесты команды `lltt post` через публичный `run`.

mod common;

use liveletters_post::{Args, NewArgs, PostAction, run};

#[test]
fn post_new_creates_persisted_post_with_default_visibility() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("alice");

    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Текст первой записи").unwrap();

    let args = Args {
        action: PostAction::New(NewArgs {
            body_file: Some(body_path),
            visibility: "public".to_owned(),
        }),
    };

    run(&ctx, &args).expect("post new should succeed");

    let store = home.open_store();
    let posts = store.list_posts().expect("list posts");
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].body, "Текст первой записи");
    assert_eq!(posts[0].visibility, "public");
    assert_eq!(posts[0].resource_id, "alice-publish@example.org");
    assert_eq!(posts[0].author_id, "alice");
}

#[test]
fn post_new_with_friends_only_visibility() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("alice");

    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Только для друзей").unwrap();

    let args = Args {
        action: PostAction::New(NewArgs {
            body_file: Some(body_path),
            visibility: "friends_only".to_owned(),
        }),
    };

    run(&ctx, &args).expect("post new should succeed");

    let store = home.open_store();
    let posts = store.list_posts().expect("list posts");
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].visibility, "friends_only");
}

#[test]
fn post_new_rejects_empty_body() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("alice");

    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "   \n  ").unwrap();

    let args = Args {
        action: PostAction::New(NewArgs {
            body_file: Some(body_path),
            visibility: "public".to_owned(),
        }),
    };

    let err = run(&ctx, &args).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("пустое") || msg.contains("пуст"), "got: {msg}");
}

#[test]
fn post_new_rejects_unknown_visibility() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("alice");

    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Текст").unwrap();

    let args = Args {
        action: PostAction::New(NewArgs {
            body_file: Some(body_path),
            visibility: "members_only".to_owned(),
        }),
    };

    let err = run(&ctx, &args).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("видимости") || msg.contains("visibility"),
        "got: {msg}"
    );
}
