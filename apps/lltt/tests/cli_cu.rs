//! Интеграционные тесты команды `lltt cu` через бинарь.

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use liveletters_store::{PostRecord, Store};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn write_identity(home: &TempDir, name: &str) {
    fs::create_dir_all(home.path().join("identities")).unwrap();
    fs::write(
        home.path().join("identities").join(format!("{name}.toml")),
        format!(
            r#"
display_name = "Тест {name}"

[mail]
publish = "https://example.com/{name}/"
receive = ["comments+{name}@example.com"]
"#
        ),
    )
    .unwrap();
}

#[test]
fn user_list_is_empty_after_init() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn cu_with_no_args_errors_before_current_user_selected() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("cu")
        .assert()
        .failure()
        .code(2)
        .stderr(contains("lltt user init"));
}

#[test]
fn cu_switch_writes_current_user_file() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let cu = fs::read_to_string(tmp.path().join("current-user")).unwrap();
    assert_eq!(cu.trim(), "alice");
}

#[test]
fn cu_switch_errors_on_unknown_identity() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("cu")
        .arg("ghost")
        .assert()
        .failure();
}

#[test]
fn cu_show_prints_identity() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("cu")
        .arg("show")
        .assert()
        .success();
}

#[test]
fn cu_show_masks_smtp_password_in_stdout() {
    let tmp = TempDir::new().unwrap();
    let password = "supersecretpw9";
    common::init_user(tmp.path(), "alice");
    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store
        .save_identity(
            "alice",
            "alice@example.org",
            "Alice",
            Some("https://example.com/alice/"),
            "ru",
            true,
        )
        .unwrap();
    store
        .save_mail_settings_record(&liveletters_store::MailSettingsRecord {
            profile_id: "alice".into(),
            smtp_host: "smtp.example.com".into(),
            smtp_port: 465,
            smtp_security: "tls".into(),
            smtp_username: "alice@example.com".into(),
            smtp_password: password.into(),
            smtp_hello_domain: "example.com".into(),
            imap_host: "imap.example.com".into(),
            imap_port: 993,
            imap_security: "tls".into(),
            imap_username: "alice@example.com".into(),
            imap_password: password.into(),
            imap_mailbox: "INBOX".into(),
            initial_lookback_days: 1,
        })
        .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("show")
        .arg("alice")
        .assert()
        .success()
        .stdout(contains("********"))
        .stdout(contains(password).not());
}

#[test]
fn cu_user_add_show_masks_passwords() {
    let tmp = TempDir::new().unwrap();
    let password = "topsecret42";
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["init", "--force"])
        .assert()
        .success();
    fs::create_dir_all(tmp.path().join("drafts")).unwrap();
    fs::write(
        tmp.path().join("drafts/alice.toml"),
        format!(
            r#"
display_name = "Alice"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice@example.org"
password = "{password}"
hello_domain = "example.org"
pwd_obfuscate = false

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "alice@example.org"
password = "{password}"
mailbox = "INBOX"
pwd_obfuscate = false
"#
        ),
    )
    .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice", "--yes"])
        .assert()
        .success();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("show")
        .arg("alice")
        .assert()
        .success()
        .stdout(contains("********"))
        .stdout(contains(password).not());
}

#[test]
fn cu_user_add_yes_obfuscates_passwords_without_prompting_again() {
    let tmp = TempDir::new().unwrap();
    let smtp_password = "smtp-secret-42";
    let imap_password = "imap-secret-43";
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["init", "--force"])
        .assert()
        .success();
    fs::create_dir_all(tmp.path().join("drafts")).unwrap();
    let draft = tmp.path().join("drafts/alice.toml");
    fs::write(
        &draft,
        format!(
            r#"
display_name = "Alice"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice@example.org"
password = "{smtp_password}"
hello_domain = "example.org"
pwd_obfuscate = true

[mail.imap]
host = "imap.example.org"
port = 993
security = "tls"
username = "alice@example.org"
password = "{imap_password}"
mailbox = "INBOX"
pwd_obfuscate = true
"#
        ),
    )
    .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["user", "add", "alice", "--yes"])
        .assert()
        .success();

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    let settings = store.get_mail_settings_record("alice").unwrap().unwrap();
    assert_eq!(settings.smtp_password, smtp_password);
    assert_eq!(settings.imap_password, imap_password);

    let rewritten = fs::read_to_string(draft).unwrap();
    assert!(rewritten.contains("pwd_obfuscate = true"));
    assert!(
        !rewritten.contains(smtp_password) && !rewritten.contains(imap_password),
        "черновик должен хранить скрытые, а не открытые пароли: {rewritten}"
    );
}

#[test]
fn cu_user_add_short_yes_obfuscates_passwords_without_prompting_again() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["init", "--force"])
        .assert()
        .success();
    let from = tmp.path().join("alice.toml");
    fs::write(
        &from,
        r#"
display_name = "Alice"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]

[mail.smtp]
host = "smtp.example.org"
port = 587
security = "starttls"
username = "alice@example.org"
password = "smtp-secret"
hello_domain = "example.org"
pwd_obfuscate = true
"#,
    )
    .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("add")
        .arg("alice")
        .arg("--from")
        .arg(&from)
        .arg("-y")
        .assert()
        .success();

    let rewritten = fs::read_to_string(from).unwrap();
    assert!(!rewritten.contains("smtp-secret"));
}

#[test]
fn cu_add_creates_identity_file() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
    let from = tmp.path().join("source.toml");
    fs::write(
        &from,
        r#"
display_name = "Каролина"

[mail]
publish = "carol@example.com"
"#,
    )
    .unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("add")
        .arg("carol")
        .arg("--from")
        .arg(&from)
        .assert()
        .success();
    let store = Store::open_for_home_dir(tmp.path().join("users/carol")).unwrap();
    assert!(store.get_user_settings_record("carol").unwrap().is_some());
}

#[test]
fn cu_rm_requires_yes() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("rm")
        .arg("alice")
        .assert()
        .failure();
    assert!(tmp.path().join("users/alice/liveletters.sqlite3").exists());
}

#[test]
fn cu_rm_with_yes_deletes_identity() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    write_identity(&tmp, "bob");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["set", "language", "ru"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("rm")
        .arg("alice")
        .arg("--yes")
        .assert()
        .success();
    assert!(!tmp.path().join("users/alice/liveletters.sqlite3").exists());
}

#[test]
fn old_cu_management_commands_are_rejected_with_user_hint() {
    let tmp = TempDir::new().unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();

    for args in [
        vec!["cu", "list"],
        vec!["cu", "show", "alice"],
        vec!["cu", "add", "alice", "--from", "alice.toml"],
        vec!["cu", "rm", "alice", "--yes"],
    ] {
        lltt()
            .env("LIVELETTERS_HOME", tmp.path())
            .args(args)
            .assert()
            .failure()
            .stderr(contains("lltt user"));
    }
}

#[test]
fn cu_posts_prints_current_users_posts_newest_first() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    for (post_id, author_id, created_at) in [
        ("old-alice", "alice@example.org", 1_710_000_000),
        ("new-alice", "alice@example.org", 1_710_000_100),
        ("bob-post", "bob@example.org", 1_710_000_200),
    ] {
        let resource = format!("{author_id}-blog");
        store.save_author(&resource, &resource, "test").unwrap();
        store.save_author(author_id, author_id, "test").unwrap();
        store
            .save_post_record(&PostRecord {
                post_id: post_id.into(),
                resource_email: resource,
                author_email: author_id.into(),
                created_at,
                body: post_id.into(),
                visibility: "public".into(),
                hidden: false,
            })
            .unwrap();
    }

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "posts", "--limit", "1"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(stdout.contains("new-alice"), "stdout = {stdout}");
    assert!(!stdout.contains("old-alice"), "stdout = {stdout}");
    assert!(!stdout.contains("bob-post"), "stdout = {stdout}");
}

#[test]
fn cu_posts_prints_fresh_author_identity_from_authors_table() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store
        .save_author("alice@example.org", "Alice", "test")
        .unwrap();
    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_email: "alice@example.org".into(),
            author_email: "alice@example.org".into(),
            created_at: 1_710_000_000,
            body: "Мой пост".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    store
        .save_author("alice@example.org", "Robert", "test")
        .unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "posts"])
        .assert()
        .success()
        .stdout(contains("Robert <alice@example.org>"));
}

#[test]
fn cu_posts_works_with_db_only_identity_no_toml() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    // удаляем TOML — identity должна работать только из базы
    let tom_path = tmp.path().join("identities/alice.toml");
    if tom_path.exists() {
        std::fs::remove_file(&tom_path).unwrap();
    }

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    store
        .save_author("alice-blog", "alice-blog", "test")
        .unwrap();
    store
        .save_author("alice@example.org", "alice", "test")
        .unwrap();
    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_email: "alice-blog".into(),
            author_email: "alice@example.org".into(),
            created_at: 1_710_000_000,
            body: "Мой пост".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "posts"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Мой пост"),
        "cu posts should show posts with DB-only identity (no TOML): {stdout}"
    );
}
