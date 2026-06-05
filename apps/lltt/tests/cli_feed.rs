//! Интеграционные тесты команды `lltt feed` через бинарь.

mod common;

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn init_home(tmp: &TempDir) {
    common::init_user(tmp.path(), "alice");
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
fn feed_after_inbox_import_prints_post() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let eml = common::write_post_eml(tmp.path(), "post_abc", "Привет из письма");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("import")
        .arg(&eml)
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .assert()
        .success()
        .stdout(contains("пост #post_abc"));
}

#[test]
fn feed_with_limit_truncates() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    for id in ["a", "b", "c"] {
        let eml = common::write_post_eml(tmp.path(), id, "тело");
        lltt()
            .env("LIVELETTERS_HOME", tmp.path())
            .arg("inbox")
            .arg("import")
            .arg(&eml)
            .assert()
            .success();
    }
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("feed")
        .arg("--limit")
        .arg("2")
        .assert()
        .success()
        .stdout(contains("(показано: 2)"));
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
