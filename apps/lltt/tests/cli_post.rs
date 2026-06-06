//! Интеграционные тесты команды `lltt post` через бинарь.

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

#[test]
fn post_new_creates_persisted_post() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    let body_path = tmp.path().join("body.txt");
    fs::write(&body_path, "Тело первой записи").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "post",
            "new",
            "--body-file",
            body_path.to_str().unwrap(),
            "--visibility",
            "public",
        ])
        .assert()
        .success()
        .stdout(contains("запись создана:"));

    let db =
        rusqlite::Connection::open(tmp.path().join("users/alice/liveletters.sqlite3")).unwrap();
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM posts", [], |r| r.get::<_, i64>(0))
        .unwrap();
    assert_eq!(count, 1);
    let visibility: String = db
        .query_row("SELECT visibility FROM posts LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();
    assert_eq!(visibility, "public");
}

#[test]
fn post_new_with_friends_only_visibility() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    let body_path = tmp.path().join("body.txt");
    fs::write(&body_path, "Только для друзей").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "post",
            "new",
            "--body-file",
            body_path.to_str().unwrap(),
            "--visibility",
            "friends_only",
        ])
        .assert()
        .success();

    let db =
        rusqlite::Connection::open(tmp.path().join("users/alice/liveletters.sqlite3")).unwrap();
    let visibility: String = db
        .query_row("SELECT visibility FROM posts LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();
    assert_eq!(visibility, "friends_only");
}
