//! Интеграционные тесты команды `lltt sub` через бинарь.

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
fn sub_subscribe_writes_toml_and_outbox() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("alice-publish@example.org")
        .assert()
        .success();

    let text = fs::read_to_string(tmp.path().join("identities/bob.toml")).unwrap();
    assert!(text.contains("alice-publish@example.org"));
    assert!(tmp.path().join("liveletters.sqlite3").exists());
}

#[test]
fn sub_subscribe_rejects_invalid_address() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("not-an-address")
        .assert()
        .failure();
}

#[test]
fn sub_list_succeeds_after_subscribe() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("alice-publish@example.org")
        .assert()
        .success();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("list")
        .assert()
        .success()
        .stdout(contains("alice-publish@example.org"));
}

#[test]
fn sub_rm_removes_subscription() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("alice-publish@example.org")
        .assert()
        .success();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("rm")
        .arg("alice-publish@example.org")
        .assert()
        .success();

    let text = fs::read_to_string(tmp.path().join("identities/bob.toml")).unwrap();
    assert!(!text.contains("alice-publish@example.org"));
}
