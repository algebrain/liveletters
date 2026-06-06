//! Интеграционные тесты команды `lltt cu` через бинарь.

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use liveletters_store::{PostRecord, Store};
use predicates::prelude::PredicateBooleanExt;
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

fn write_identity(home: &TempDir, name: &str) {
    fs::write(
        home.path().join("identities").join(format!("{name}.toml")),
        format!(
            r#"
account_id = "{name}"
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
    init_home(&tmp);
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
    init_home(&tmp);
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
    init_home(&tmp);
    write_identity(&tmp, "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("cu")
        .arg("alice")
        .assert()
        .success();
    let cu = fs::read_to_string(tmp.path().join("current-user")).unwrap();
    assert_eq!(cu.trim(), "alice");
}

#[test]
fn cu_switch_errors_on_unknown_identity() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
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
    init_home(&tmp);
    write_identity(&tmp, "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("cu")
        .arg("alice")
        .assert()
        .success();
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
    init_home(&tmp);
    let password = "supersecretpw9";
    fs::write(
        tmp.path().join("identities").join("alice.toml"),
        format!(
            r#"
account_id = "alice"
display_name = "Alice"

[mail]
publish = "https://example.com/alice/"
receive = ["comments+alice@example.com"]

[mail.smtp]
host = "smtp.example.com"
port = 465
security = "tls"
username = "alice@example.com"
password = "{password}"

[mail.imap]
host = "imap.example.com"
port = 993
security = "tls"
username = "alice@example.com"
password = "{password}"
mailbox = "INBOX"
"#
        ),
    )
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
fn cu_add_creates_identity_file() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    let from = tmp.path().join("source.toml");
    fs::write(
        &from,
        r#"
account_id = "carol"
display_name = "Каролина"

[mail]
publish = "https://example.com/carol/"
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
    assert!(tmp.path().join("identities/carol.toml").exists());
}

#[test]
fn cu_rm_requires_yes() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    write_identity(&tmp, "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("rm")
        .arg("alice")
        .assert()
        .failure();
    assert!(tmp.path().join("identities/alice.toml").exists());
}

#[test]
fn cu_rm_with_yes_deletes_identity() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);
    write_identity(&tmp, "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("user")
        .arg("rm")
        .arg("alice")
        .arg("--yes")
        .assert()
        .success();
    assert!(!tmp.path().join("identities/alice.toml").exists());
}

#[test]
fn old_cu_management_commands_are_rejected_with_user_hint() {
    let tmp = TempDir::new().unwrap();
    init_home(&tmp);

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
    init_home(&tmp);
    write_identity(&tmp, "alice");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "alice"])
        .assert()
        .success();

    let store = Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    for (post_id, author_id, created_at) in [
        ("old-alice", "alice", 1_710_000_000),
        ("new-alice", "alice", 1_710_000_100),
        ("bob-post", "bob", 1_710_000_200),
    ] {
        store
            .save_post_record(&PostRecord {
                post_id: post_id.into(),
                resource_id: format!("{author_id}-blog"),
                author_id: author_id.into(),
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
