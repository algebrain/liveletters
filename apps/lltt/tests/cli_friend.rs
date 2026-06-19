//! Интеграционные тесты команды `lltt friend` через бинарь.

use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

#[test]
fn friend_without_existing_subscription_creates_pending_friend_and_plain_subscription_request() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["friend", "bob@example.org"])
        .assert()
        .success()
        .stdout(contains("запрошено добавление в друзья: bob@example.org"));

    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    let pending_friends = store.list_pending_friends("alice").unwrap();
    assert_eq!(pending_friends.len(), 1);
    assert_eq!(pending_friends[0].owner_resource_email, "alice@example.org");
    assert_eq!(pending_friends[0].friend_email, "bob@example.org");
    assert_eq!(
        pending_friends[0].subscribed_resource_email,
        "bob@example.org"
    );
    assert!(
        !store
            .is_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );

    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_type, "subscription_requested");
    let json: serde_json::Value = serde_json::from_str(&outbox[0].message_body).unwrap();
    assert!(
        json["payload"].get("purpose").is_none(),
        "friend не должен добавлять purpose в SubscriptionRequested: {json}"
    );
}

#[test]
fn friend_with_existing_subscription_sends_friend_added_and_sub_list_reports_it() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store.save_author("bob@example.org", "bob", "test").unwrap();
    store
        .add_local_subscription("alice", "bob@example.org")
        .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["friend", "bob@example.org"])
        .assert()
        .success()
        .stdout(contains("добавлен в друзья: bob@example.org"));

    assert!(
        store
            .is_friend("alice@example.org", "bob@example.org")
            .unwrap()
    );
    let outbox = store.list_outbox_records().unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_type, "friend_added");
    assert!(
        outbox[0]
            .human_readable_body
            .as_deref()
            .unwrap_or("")
            .contains("друз")
    );

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sub", "list"])
        .assert()
        .success()
        .stdout(contains("мои друзья:"))
        .stdout(contains("bob@example.org"))
        .stdout(contains("я в друзьях у:"));
}
