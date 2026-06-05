//! Интеграционные тесты команды `lltt inbox import` через бинарь.

mod common;

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn init_home(tmp: &TempDir) {
    common::init_user(tmp.path(), "alice");
}

#[test]
fn inbox_import_inserts_post_into_store() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let eml = common::write_post_eml(tmp.path(), "post_xyz", "тело");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("import")
        .arg(&eml)
        .assert()
        .success()
        .stdout(contains("применено: 1"));
}

#[test]
fn inbox_import_repeated_yields_duplicate() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let eml = common::write_post_eml(tmp.path(), "post_xyz", "тело");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("import")
        .arg(&eml)
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("import")
        .arg(&eml)
        .assert()
        .success()
        .stdout(contains("дубликат"));
}

#[test]
fn inbox_import_missing_file_returns_error() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let missing = tmp.path().join("nonexistent.eml");
    fs::write(&missing, b"").ok();
    let _ = fs::remove_file(&missing);
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("import")
        .arg(&missing)
        .assert()
        .failure()
        .stderr(contains("не найден").or(contains("not found")));
}
