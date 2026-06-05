//! Проверяет команду `lltt doctor --verbose` через бинарь (этап 8.28).

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn doctor_verbose_after_init_shows_sections() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["doctor", "--verbose"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("--- identities ---"), "stdout = {stdout}");
    assert!(stdout.contains("--- таблицы ---"), "stdout = {stdout}");
    assert!(stdout.contains("posts: "), "stdout = {stdout}");
    assert!(stdout.contains(" B"), "stdout = {stdout}");
}

#[test]
fn doctor_verbose_off_matches_legacy_output() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");
    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .arg("doctor")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("здоровье:"), "stdout = {stdout}");
    assert!(!stdout.contains("--- identities ---"), "stdout = {stdout}");
    assert!(!stdout.contains("--- таблицы ---"), "stdout = {stdout}");
}
