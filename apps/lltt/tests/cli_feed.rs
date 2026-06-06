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
}

fn subscribe_alice_to(tmp: &TempDir, resource: &str) {
    std::fs::write(
        tmp.path().join("identities/alice.toml"),
        format!(
            r#"
account_id = "acct_alice"
display_name = "alice"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]

[meta]
resources_owned = ["alice@example.org"]
subscriptions = ["{resource}"]
"#
        ),
    )
    .unwrap();
}

fn save_post(tmp: &TempDir, post_id: &str, resource_id: &str, author_id: &str, created_at: u64) {
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    store
        .save_post_record(&PostRecord {
            post_id: post_id.into(),
            resource_id: resource_id.into(),
            author_id: author_id.into(),
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

    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    store
        .save_post_record(&PostRecord {
            post_id: "own-post".into(),
            resource_id: "alice@example.org".into(),
            author_id: "acct_alice".into(),
            created_at: 1_710_000_000,
            body: "мой пост".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

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
    save_post(&tmp, "old", "bob@example.org", "bob", 1_710_000_000);
    save_post(&tmp, "middle", "bob@example.org", "bob", 1_710_000_100);
    save_post(&tmp, "new", "bob@example.org", "bob", 1_710_000_200);

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
        "acct_alice",
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
fn feed_on_missing_init_returns_error() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .failure();
}
