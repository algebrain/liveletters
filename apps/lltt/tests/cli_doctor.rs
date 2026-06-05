//! Проверяет команду `lltt doctor` через бинарь.

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn doctor_prints_health_summary() {
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
    assert!(stdout.contains("Applied:"), "stdout = {stdout}");
}

#[test]
fn doctor_reports_degraded_after_malformed_import() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().to_path_buf();
    common::init_user(&home, "alice");

    let bad = common::write_malformed_post_eml(&home, "bad-1");
    lltt()
        .env("LIVELETTERS_HOME", &home)
        .args(["inbox", "import", bad.to_str().unwrap()])
        .assert()
        .success();

    let assert = lltt()
        .env("LIVELETTERS_HOME", &home)
        .arg("doctor")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("здоровье: деградирован"),
        "stdout = {stdout}"
    );
    assert!(stdout.contains("Malformed: 1"), "stdout = {stdout}");
}
