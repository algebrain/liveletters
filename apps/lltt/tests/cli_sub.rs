//! Интеграционные тесты команды `lltt sub` через бинарь.

use std::process::Command;

use assert_cmd::prelude::*;
use liveletters_store::Store;
use predicates::str::contains;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn local_subscriptions(tmp: &TempDir, name: &str) -> Vec<String> {
    Store::open_for_home_dir(tmp.path().join("users").join(name))
        .unwrap()
        .list_local_subscriptions(name)
        .unwrap()
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

    let subs = local_subscriptions(&tmp, "bob");
    assert!(subs.contains(&"alice-publish@example.org".to_owned()));
    assert!(tmp.path().join("users/bob/liveletters.sqlite3").exists());
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

    let subs = local_subscriptions(&tmp, "bob");
    assert!(!subs.contains(&"alice-publish@example.org".to_owned()));
}
