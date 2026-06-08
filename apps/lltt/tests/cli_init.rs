//! Интеграционные тесты команды `lltt init` через бинарь.

use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

#[test]
fn init_creates_liveletters_sqlite3() {
    let tmp = TempDir::new().expect("tempdir");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();

    assert!(tmp.path().join("liveletters.sqlite3").exists());
    assert!(tmp.path().join("mail-password-obfuscation.key").exists());
    assert!(tmp.path().join("drafts").is_dir());
    assert!(!tmp.path().join("identities/default.toml").exists());
    assert!(!tmp.path().join("current-user").exists());
}

#[test]
fn init_fails_on_non_empty_home_without_force() {
    let tmp = TempDir::new().expect("tempdir");
    std::fs::write(tmp.path().join("junk"), b"x").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("уже существует"));
}

#[test]
fn init_force_succeeds_on_non_empty_home() {
    let tmp = TempDir::new().expect("tempdir");
    std::fs::write(tmp.path().join("junk"), b"x").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["init", "--force"])
        .assert()
        .success();
}
