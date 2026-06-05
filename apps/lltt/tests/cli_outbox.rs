//! Интеграционные тесты команды `lltt outbox` через бинарь.

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
fn outbox_list_empty_store_succeeds() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["outbox", "list"])
        .assert()
        .success()
        .stdout(contains("неотправленные события: 0"));
}

#[test]
fn outbox_list_after_post_creation_contains_post_created() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    let body_path = tmp.path().join("body.txt");
    fs::write(&body_path, "Запись").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["outbox", "list"])
        .assert()
        .success()
        .stdout(contains("post_created"));
}
