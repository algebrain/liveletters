//! Интеграционные тесты команды `lltt feed` через бинарь.

mod common;

use std::process::Command;

use assert_cmd::prelude::*;
use liveletters_store::{PostRecord, Store};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn init_home(tmp: &TempDir) {
    common::init_user(tmp.path(), "alice");
    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store
        .save_author("alice@example.org", "alice", "test")
        .unwrap();
    store
        .save_resources_owned("alice", &["alice@example.org".to_owned()])
        .unwrap();
}

fn subscribe_alice_to(tmp: &TempDir, resource: &str) {
    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store.save_author(resource, resource, "test").unwrap();
    store.add_local_subscription("alice", resource).unwrap();
}

fn save_post(tmp: &TempDir, post_id: &str, resource_id: &str, author_id: &str, created_at: u64) {
    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store.save_author(resource_id, resource_id, "test").unwrap();
    store.save_author(author_id, author_id, "test").unwrap();
    store
        .save_post_record(&PostRecord {
            post_id: post_id.into(),
            resource_email: resource_id.into(),
            author_email: author_id.into(),
            created_at,
            body: post_id.into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
}

#[test]
fn feed_on_empty_home_prints_empty_marker() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .success()
        .stdout(contains("(пусто)"));
}

#[test]
fn feed_no_longer_shows_current_users_own_posts() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);

    save_post(
        &tmp,
        "own-post",
        "alice@example.org",
        "alice@example.org",
        1_710_000_000,
    );

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(!stdout.contains("own-post"), "stdout = {stdout}");
}

#[test]
fn feed_with_limit_truncates() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    subscribe_alice_to(&tmp, "bob@example.org");
    save_post(
        &tmp,
        "old",
        "bob@example.org",
        "bob@example.org",
        1_710_000_000,
    );
    save_post(
        &tmp,
        "middle",
        "bob@example.org",
        "bob@example.org",
        1_710_000_100,
    );
    save_post(
        &tmp,
        "new",
        "bob@example.org",
        "bob@example.org",
        1_710_000_200,
    );

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .arg("--limit")
        .arg("2")
        .assert()
        .success()
        .stdout(contains("(показано: 2)"))
        .stdout(contains("new"))
        .stdout(contains("middle"))
        .stdout(contains("old").not());
}

#[test]
fn feed_shows_subscribed_posts_and_hides_unsubscribed_posts() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    subscribe_alice_to(&tmp, "bob@example.org");

    save_post(
        &tmp,
        "own",
        "alice@example.org",
        "alice@example.org",
        1_710_000_300,
    );
    save_post(&tmp, "subscribed", "bob@example.org", "bob", 1_710_000_200);
    save_post(
        &tmp,
        "unsubscribed",
        "carol@example.org",
        "carol",
        1_710_000_100,
    );

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .success()
        .stdout(contains("subscribed"))
        .stdout(contains("own").not())
        .stdout(contains("unsubscribed").not());
}

#[test]
fn feed_prints_fresh_author_identity_from_authors_table() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    subscribe_alice_to(&tmp, "bob@example.org");
    save_post(
        &tmp,
        "subscribed",
        "bob@example.org",
        "bob@example.org",
        1_710_000_200,
    );

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store
        .save_author("bob@example.org", "Robert", "test")
        .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .success()
        .stdout(contains("Robert <bob@example.org>"));
}

#[test]
fn feed_does_not_read_posts_from_shared_home_database() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "algebrain");
    // create austin identity via DB
    common::write_identity(tmp.path(), "austin");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "austin"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["set", "language", "ru"])
        .assert()
        .success();
    // subscribe austin to algebrain
    let austin_store = Store::open_for_home_dir(tmp.path().join("users/austin")).unwrap();
    austin_store
        .save_author("algebrain@example.org", "algebrain", "test")
        .unwrap();
    austin_store
        .save_author("austin@example.org", "austin", "test")
        .unwrap();
    austin_store
        .add_local_subscription("austin", "algebrain@example.org")
        .unwrap();
    austin_store
        .save_resources_owned("austin", &["austin@example.org".to_owned()])
        .unwrap();

    let shared_store = Store::open_for_home_dir(tmp.path()).unwrap();
    shared_store
        .save_author("algebrain@example.org", "algebrain", "test")
        .unwrap();
    shared_store
        .save_post_record(&PostRecord {
            post_id: "shared-db-post".into(),
            resource_email: "algebrain@example.org".into(),
            author_email: "algebrain@example.org".into(),
            created_at: 1_710_000_400,
            body: "shared-db-post".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .success()
        .stdout(contains("shared-db-post").not());
}

#[test]
fn feed_on_missing_init_returns_error() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .failure();
}
