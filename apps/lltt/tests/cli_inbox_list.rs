//! Проверяет команду `lltt inbox list` через бинарь.

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn inbox_list_prints_empty_when_no_messages() {
    let tmp = TempDir::new().expect("tempdir");
    common::init_user(tmp.path(), "alice");
    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("list")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("входящих всего: 0"), "stdout = {stdout}");
    assert!(stdout.contains("(пусто)"), "stdout = {stdout}");
}

#[test]
fn inbox_list_with_status_filter() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");
    let eml = common::write_post_eml(&home, "post-1", "Привет, мир");
    lltt()
        .env("LIVELETTERS_HOME", &home)
        .arg("inbox")
        .arg("import")
        .arg(&eml)
        .assert()
        .success();
    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .arg("inbox")
        .arg("list")
        .arg("--status")
        .arg("applied")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("applied"), "stdout = {stdout}");
    assert!(stdout.contains("показано: 1"), "stdout = {stdout}");
    assert!(
        stdout.contains("post-1@example.test"),
        "message_id отсутствует: {stdout}"
    );
}

#[test]
fn inbox_list_rejects_unknown_status() {
    let tmp = TempDir::new().expect("tempdir");
    common::init_user(tmp.path(), "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("inbox")
        .arg("list")
        .arg("--status")
        .arg("nonsense")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn inbox_list_respects_limit() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    for i in 0..3 {
        let eml = common::write_post_eml(&home, &format!("p-{i}"), &format!("body {i}"));
        lltt()
            .env("LIVELETTERS_HOME", &home)
            .args(["inbox", "import", eml.to_str().unwrap()])
            .assert()
            .success();
    }

    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["inbox", "list", "--limit", "2"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("показано: 2"), "stdout = {stdout}");
}

#[test]
fn inbox_list_without_init_returns_code_2() {
    let tmp = TempDir::new().expect("tempdir");
    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["inbox", "list"])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("current-user") || stderr.contains("текущ"),
        "stderr = {stderr}"
    );
}
