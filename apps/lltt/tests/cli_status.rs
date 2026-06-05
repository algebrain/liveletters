//! Проверяет команду `lltt status` через бинарь.

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn status_prints_counts_on_fresh_home() {
    let tmp = TempDir::new().expect("tempdir");
    common::init_user(tmp.path(), "alice");
    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("status")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("постов:"), "stdout = {stdout}");
    assert!(stdout.contains("комментариев:"), "stdout = {stdout}");
    assert!(stdout.contains("нет активности"), "stdout = {stdout}");
}
