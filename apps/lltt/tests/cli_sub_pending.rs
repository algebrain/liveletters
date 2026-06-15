//! CLI-тесты `lltt sub pending` и `lltt sub cancel`.

use assert_cmd::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    assert_cmd::Command::cargo_bin("lltt").expect("lltt binary")
}

#[test]
fn sub_pending_lists_outstanding_subscriptions() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "alice-publish@example.org"])
        .assert()
        .success();

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "pending"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("alice-publish@example.org"),
        "pending должен показывать alice-publish@example.org:\n{stdout}"
    );
}

#[test]
fn sub_pending_is_empty_before_any_subscribe() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "pending"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("пусто"),
        "pending должен быть пуст:\n{stdout}"
    );
}

#[test]
fn sub_cancel_removes_pending_subscription() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "alice-publish@example.org"])
        .assert()
        .success();

    // До отмены: pending есть
    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "pending"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("alice-publish@example.org"));

    // Отменяем
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "cancel", "alice-publish@example.org"])
        .assert()
        .success();

    // После отмены: pending пуст
    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "pending"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains("alice-publish@example.org"),
        "alice-publish@example.org не должно быть в pending:\n{stdout}"
    );
}

#[test]
fn repeated_sub_does_not_duplicate_pending() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "bob");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "alice-publish@example.org"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "alice-publish@example.org"])
        .assert()
        .success();

    // pending содержит только одну запись — повторный sub не должен
    // дублировать её
    let store = liveletters_store::Store::open_for_home_dir(tmp.path().join("users/bob")).unwrap();
    let pending = store.list_pending_subscriptions("bob").unwrap();
    assert_eq!(pending.len(), 1, "не должно быть дубликатов в pending");
}
