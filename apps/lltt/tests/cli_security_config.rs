//! Интеграционный тест: `lltt user add` создаёт per-user `config.toml`
//! с настройками безопасности (MimeLimits/IngestLimits/RetentionPolicy).

use std::{fs, process::Command};

use assert_cmd::prelude::*;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn write_alice_draft(tmp: &TempDir) {
    let source = tmp.path().join("alice.toml");
    fs::write(
        &source,
        r#"
display_name = "Алиса"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice@example.org"
password = ""
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "alice@example.org"
password = ""
mailbox = "INBOX"
"#,
    )
    .unwrap();
}

#[test]
fn user_add_creates_per_user_security_config() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
    write_alice_draft(&tmp);

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice", "--from"])
        .arg(tmp.path().join("alice.toml"))
        .assert()
        .success();

    let cfg_path = tmp.path().join("users/alice/config.toml");
    assert!(
        cfg_path.exists(),
        "config.toml должен быть создан при user add"
    );

    // Файл парсится в SecurityConfig (через официальный loader) и содержит
    // все три секции с defaults.
    let parsed = liveletters_config::SecurityConfig::load(&tmp.path().join("users/alice")).unwrap();
    assert_eq!(parsed, liveletters_config::SecurityConfig::default());

    let raw = fs::read_to_string(&cfg_path).unwrap();
    assert!(raw.contains("[ingest_limits]"));
    assert!(raw.contains("[mime_limits]"));
    assert!(raw.contains("[retention]"));
}
