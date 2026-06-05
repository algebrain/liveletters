//! Интеграционные тесты команды `lltt comment` через бинарь.

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn create_post(tmp: &TempDir) -> String {
    let body_path = tmp.path().join("body.txt");
    fs::write(&body_path, "Запись").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let db = rusqlite::Connection::open(tmp.path().join("liveletters.sqlite3")).unwrap();
    db.query_row("SELECT post_id FROM posts LIMIT 1", [], |r| {
        r.get::<_, String>(0)
    })
    .unwrap()
}

#[test]
fn comment_new_creates_persisted_comment() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    let body_path = tmp.path().join("c.txt");
    fs::write(&body_path, "Первый комментарий").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "comment",
            "new",
            "--post",
            &post_id,
            "--body-file",
            body_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("комментарий создан:"));

    let db = rusqlite::Connection::open(tmp.path().join("liveletters.sqlite3")).unwrap();
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM comments", [], |r| r.get::<_, i64>(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn comment_new_with_parent_creates_reply() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    let root_body = tmp.path().join("root.txt");
    fs::write(&root_body, "Корневой").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "comment",
            "new",
            "--post",
            &post_id,
            "--body-file",
            root_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    let db = rusqlite::Connection::open(tmp.path().join("liveletters.sqlite3")).unwrap();
    let root_id: String = db
        .query_row("SELECT comment_id FROM comments LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();

    let reply_body = tmp.path().join("reply.txt");
    fs::write(&reply_body, "Ответ").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "comment",
            "new",
            "--post",
            &post_id,
            "--parent",
            &root_id,
            "--body-file",
            reply_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    let parent_id: Option<String> = db
        .query_row(
            "SELECT parent_comment_id FROM comments WHERE body = 'Ответ'",
            [],
            |r| r.get::<_, Option<String>>(0),
        )
        .unwrap();
    assert_eq!(parent_id.as_deref(), Some(root_id.as_str()));
}
