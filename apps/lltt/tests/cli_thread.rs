//! Интеграционные тесты команды `lltt thread` через бинарь.

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
    fs::write(&body_path, "Запись для thread").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let db =
        rusqlite::Connection::open(tmp.path().join("users/alice/liveletters.sqlite3")).unwrap();
    db.query_row("SELECT post_id FROM posts LIMIT 1", [], |r| {
        r.get::<_, String>(0)
    })
    .unwrap()
}

#[test]
fn thread_for_existing_post_succeeds() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["thread", &post_id])
        .assert()
        .success()
        .stdout(contains(format!("пост #{}", post_id)));
}

#[test]
fn thread_with_comment_shows_comment_in_output() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    let body_path = tmp.path().join("c.txt");
    fs::write(&body_path, "Текст комментария").unwrap();
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
        .success();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["thread", &post_id])
        .assert()
        .success()
        .stdout(contains("Текст комментария"));
}
