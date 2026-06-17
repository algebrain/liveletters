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

    assert!(!tmp.path().join("current-user").exists());
}

#[test]
fn user_add_derives_nickname_from_publish_when_display_name_blank() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let source = tmp.path().join("alice.toml");
    // display_name пустое — должна подставиться локальная часть e-mail.
    fs::write(
        &source,
        r#"
display_name = ""

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
    let s = store
        .get_user_settings_record("alice")
        .unwrap()
        .expect("UserSettingsRecord должен быть создан");
    // Инвариант: после `lltt user add` e-mail непустой, а ник лежит в authors.
    // display_name было пустым → берём локальную часть `publish`.
    let author = store
        .get_author(&s.author_email)
        .unwrap()
        .expect("authors должен содержать добавленную идентичность");
    assert_eq!(
        author.nickname, "alice",
        "nickname должен быть извлечён из локальной части publish"
    );
    assert_eq!(s.author_email, "alice@example.org");
}

#[test]
fn user_add_rejects_empty_publish() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let source = tmp.path().join("alice.toml");
    fs::write(
        &source,
        r#"
display_name = "Алиса"

[mail]
publish = ""
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
        .failure()
        .stderr(contains("publish"));

    // БД не должна быть создана — нельзя работать без e-mail.
    let db = tmp.path().join("users/alice/liveletters.sqlite3");
    assert!(!db.exists(), "БД не должна создаваться, если e-mail пустой");
}

#[test]
fn user_add_rejects_publish_without_at_sign() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let source = tmp.path().join("alice.toml");
    fs::write(
        &source,
        r#"
display_name = "Алиса"

[mail]
publish = "no-at-sign"
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
        .failure()
        .stderr(contains("@"));
}

#[test]
fn user_add_rejects_blank_email_even_if_display_name_set() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let source = tmp.path().join("alice.toml");
    fs::write(
        &source,
        r#"
display_name = "Bob"

[mail]
publish = ""
receive = ["bob@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "bob@example.org"
password = ""
hello_domain = "example.org"

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "bob@example.org"
password = ""
mailbox = "INBOX"
"#,
    )
    .unwrap();

    // display_name есть, но e-mail пуст — это всё равно ошибка.
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice", "--from"])
        .arg(&source)
        .assert()
        .failure()
        .stderr(contains("publish"));
}
