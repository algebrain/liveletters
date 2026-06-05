//! Проверяет команду `lltt inbox show` через бинарь (этап 8.24).

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn inbox_show_prints_full_body_after_import() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    let eml = common::write_post_eml(&home, "p-1", "hello body");
    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["inbox", "import", eml.to_str().unwrap()])
        .assert()
        .success();

    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["inbox", "show", "<p-1@example.test>"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("--- body ---"), "stdout = {stdout}");
    assert!(stdout.contains("hello body"), "stdout = {stdout}");
}

#[test]
fn inbox_show_unknown_id_returns_error() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");
    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["inbox", "show", "nonexistent-id"])
        .assert()
        .failure()
        .code(1);
}
