//! Проверяет команду `lltt settings show|set` через бинарь.

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn settings_show_then_set_round_trip() {
    let tmp = TempDir::new().expect("tempdir");
    common::init_user(tmp.path(), "alice");

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("settings")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("[user_settings] отсутствует"),
        "stdout = {stdout}"
    );

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("settings")
        .arg("set")
        .arg("smtp.host")
        .arg("imap.example.org")
        .assert()
        .success();

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("settings")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("smtp.host: imap.example.org"),
        "stdout = {stdout}"
    );
}

#[test]
fn settings_set_rejects_unknown_key() {
    let tmp = TempDir::new().expect("tempdir");
    common::init_user(tmp.path(), "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("settings")
        .arg("set")
        .arg("bogus.key")
        .arg("value")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn settings_set_password_obfuscates_via_binary() {
    use rusqlite::Connection;

    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["settings", "set", "smtp.password", "s3cret-plaintext"])
        .assert()
        .success();

    let db_path = home.join("liveletters.sqlite3");
    assert!(db_path.exists(), "БД должна быть создана init-ом");
    let conn = Connection::open(&db_path).expect("open sqlite");
    let stored: String = conn
        .query_row("SELECT smtp_password FROM mail_settings LIMIT 1", [], |r| {
            r.get(0)
        })
        .expect("row exists");
    assert!(stored.starts_with("obf:v1:"), "stored = {stored}");
    assert!(
        !stored.contains("s3cret-plaintext"),
        "plaintext виден: {stored}"
    );
}
