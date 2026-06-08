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
        stdout.contains("nickname:") || stdout.contains("language:"),
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

    let db_path = home.join("users/alice/liveletters.sqlite3");
    assert!(db_path.exists(), "БД пользователя должна быть создана");
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

#[test]
fn set_alias_writes_log_level_to_global_config() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    // `lltt set log.level info` — короткая форма, должна попасть в <home>/config.toml.
    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["set", "log.level", "info"])
        .assert()
        .success();

    let config =
        std::fs::read_to_string(home.join("config.toml")).expect("config.toml должен быть создан");
    assert!(
        config.contains("level = \"info\""),
        "ожидалось `level = \"info\"` в config.toml, получили: {config}"
    );
    // Убедиться, что НЕ создался per-user файл.
    assert!(
        !home.join("users/alice/config.toml").exists(),
        "per-user config.toml не должен создаваться для log.*"
    );
}

#[test]
fn set_alias_writes_db_field() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    // `lltt set nickname …` — короткая форма для пользовательского поля.
    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["set", "nickname", "alice-nick"])
        .assert()
        .success();

    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .arg("settings")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("nickname: alice-nick"), "stdout = {stdout}");
}

#[test]
fn set_alias_rejects_unknown_key() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["set", "bogus.key", "value"])
        .assert()
        .failure()
        .code(1);
}
