//! Интеграционные тесты команды `lltt user` через бинарь.

use std::{fs, process::Command};

use assert_cmd::prelude::*;
use liveletters_store::Store;
use predicates::str::contains;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn init_home(tmp: &TempDir) {
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
}

#[test]
fn user_init_creates_draft_and_prints_it() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "init", "alice"])
        .assert()
        .success()
        .stdout(contains("drafts/alice.toml"))
        .stdout(contains("pwd_obfuscate = true"));

    let draft = tmp.path().join("drafts/alice.toml");
    assert!(draft.exists());
    let raw = fs::read_to_string(draft).unwrap();
    assert!(raw.contains("[mail.smtp]"));
    assert!(raw.contains("[mail.imap]"));
}

#[test]
fn user_init_requires_force_to_overwrite_draft() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "init", "alice"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "init", "alice"])
        .assert()
        .failure();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "init", "alice", "--force"])
        .assert()
        .success();
}

#[test]
fn user_add_does_not_select_current_user_automatically() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let source = tmp.path().join("alice.toml");
    fs::write(
        &source,
        r#"
account_id = "acct_alice"
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

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice", "--from"])
        .arg(&source)
        .assert()
        .success();

    assert!(tmp.path().join("identities/alice.toml").exists());
    assert!(!tmp.path().join("current-user").exists());

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    let mail = store
        .get_mail_settings_record("alice")
        .unwrap()
        .expect("mail settings should be copied from identity");
    assert_eq!(mail.smtp_host, "smtp.example.org");
    assert_eq!(mail.smtp_port, 587);
    assert_eq!(mail.smtp_security, "starttls");
    assert_eq!(mail.smtp_hello_domain, "example.org");
    assert_eq!(mail.imap_host, "imap.example.org");
    assert_eq!(mail.imap_port, 993);
    assert_eq!(mail.imap_mailbox, "INBOX");
}

#[test]
fn user_add_uses_default_draft_path_when_from_is_omitted() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "init", "alice"])
        .assert()
        .success();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice"])
        .assert()
        .success();

    assert!(tmp.path().join("identities/alice.toml").exists());
    assert!(!tmp.path().join("current-user").exists());
}

#[test]
fn user_add_leaves_nickname_and_email_empty_until_settings_explicitly_set() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let source = tmp.path().join("alice.toml");
    fs::write(
        &source,
        r#"
account_id = "acct_alice"
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

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice", "--from"])
        .arg(&source)
        .assert()
        .success();

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    let settings = store.get_user_settings_record("alice").unwrap();
    assert!(
        settings.is_some(),
        "RED: UserSettingsRecord should be created by `lltt user add` with display_name and publish"
    );
    let s = settings.unwrap();
    assert!(
        !s.nickname.is_empty(),
        "RED: nickname should be populated from display_name"
    );
    assert!(
        !s.email_address.is_empty(),
        "RED: email_address should be populated from mail.publish"
    );
}
