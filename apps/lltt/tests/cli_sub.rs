//! Интеграционные тесты команды `lltt sub` через бинарь.

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

#[test]
fn sub_subscribe_writes_pending_and_outbox() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sub")
        .arg("alice-publish@example.org")
        .assert()
        .success();

    // Подписка в pending, не в local — будет перемещена после SubscriptionConfirmed.
    assert!(tmp.path().join("users/bob/liveletters.sqlite3").exists());
    let store = liveletters_store::Store::open_for_home_dir(tmp.path().join("users/bob")).unwrap();
    let pending = store.list_pending_subscriptions("bob").unwrap();
    assert!(
        pending
            .iter()
            .any(|r| r.resource_address == "alice-publish@example.org")
    );
    let local = store.list_local_subscriptions("bob").unwrap();
    assert!(local.is_empty());
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

    // sub list показывает pending-подписки
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "pending"])
        .assert()
        .success()
        .stdout(contains("alice-publish@example.org"));
}

#[test]
fn sub_rm_removes_pending_subscription() {
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

    // pending очищен
    let store = liveletters_store::Store::open_for_home_dir(tmp.path().join("users/bob")).unwrap();
    let pending = store.list_pending_subscriptions("bob").unwrap();
    assert!(pending.is_empty());
    let local = store.list_local_subscriptions("bob").unwrap();
    assert!(local.is_empty());
}
