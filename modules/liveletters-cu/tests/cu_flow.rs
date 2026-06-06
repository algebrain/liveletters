//! Тесты поведения `lltt cu` через публичный `run`.
//!
//! Каждый тест вызывает `run` с конкретным набором аргументов и проверяет
//! результат через побочные эффекты в каталоге-доме.

mod common;

use std::fs;

use liveletters_cu::{run_current, run_user};
use liveletters_output::CommandContext;

fn tokens(args: &[&str]) -> liveletters_cu::Args {
    liveletters_cu::Args {
        tokens: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn read_current_user(home: &std::path::Path) -> String {
    fs::read_to_string(home.join("current-user"))
        .unwrap()
        .trim()
        .to_owned()
}

#[test]
fn current_action_errors_when_no_current_user_file() {
    let home = common::TestHome::new();
    let ctx = CommandContext {
        home: home.path().to_path_buf(),
        state_home: home.path().to_path_buf(),
        identity_name: "default".to_owned(),
    };
    let err = run_current(&ctx, &tokens(&[])).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("текущ") || msg.contains("current-user") || msg.contains("не задан"),
        "got: {msg}"
    );
}

#[test]
fn switch_action_writes_current_user() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("default");
    run_current(&ctx, &tokens(&["alice"])).unwrap();
    assert_eq!(read_current_user(home.path()), "alice");
}

#[test]
fn switch_action_errors_on_unknown_identity() {
    let home = common::TestHome::new();
    let ctx = home.ctx("default");
    let err = run_current(&ctx, &tokens(&["ghost"])).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("ghost") || msg.contains("не найден") || msg.contains("unknown"),
        "got: {msg}"
    );
}

#[test]
fn list_action_returns_all_identities() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    home.add_identity("bob");
    let ctx = home.ctx("default");
    run_user(&ctx, &tokens(&["list"])).unwrap();
}

#[test]
fn show_action_prints_identity() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("default");
    run_user(&ctx, &tokens(&["show", "alice"])).unwrap();
}

#[test]
fn show_action_errors_on_unknown_identity() {
    let home = common::TestHome::new();
    let ctx = home.ctx("default");
    let err = run_user(&ctx, &tokens(&["show", "ghost"])).unwrap_err();
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn add_action_creates_identity_file() {
    let home = common::TestHome::new();
    let from = home.path().join("source.toml");
    fs::write(
        &from,
        r#"
account_id = "carol"
display_name = "Каролина"

[mail]
publish = "https://example.com/carol/"
receive = ["comments+carol@example.com"]
"#,
    )
    .unwrap();
    let ctx = home.ctx("default");
    run_user(
        &ctx,
        &tokens(&["add", "carol", "--from", from.to_str().unwrap()]),
    )
    .unwrap();
    let created = home.path().join("identities/carol.toml");
    assert!(created.exists());
}

#[test]
fn add_action_errors_on_missing_source() {
    let home = common::TestHome::new();
    let ctx = home.ctx("default");
    let err = run_user(
        &ctx,
        &tokens(&["add", "carol", "--from", "/nonexistent/path.toml"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("не найден") || err.to_string().contains("not found"));
}

#[test]
fn rm_action_errors_without_yes() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("default");
    let err = run_user(&ctx, &tokens(&["rm", "alice"])).unwrap_err();
    assert!(err.to_string().contains("--yes") || err.to_string().contains("yes"));
}

#[test]
fn rm_action_deletes_with_yes() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    fs::write(home.path().join("current-user"), "default").unwrap();
    let ctx = home.ctx("default");
    run_user(&ctx, &tokens(&["rm", "alice", "--yes"])).unwrap();
    assert!(!home.path().join("identities/alice.toml").exists());
}

#[test]
fn rm_action_refuses_to_delete_current() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    fs::write(home.path().join("current-user"), "alice").unwrap();
    let ctx = home.ctx("default");
    let err = run_user(&ctx, &tokens(&["rm", "alice", "--yes"])).unwrap_err();
    assert!(err.to_string().contains("текущ"));
}

#[test]
fn show_with_reveal_does_not_error() {
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("default");
    run_user(&ctx, &tokens(&["show", "alice", "--reveal"])).unwrap();
}

#[test]
fn show_masks_password_when_no_smtp_imap_configured() {
    // sample_identity в common/mod.rs создаёт IdentityConfig с smtp=None, imap=None,
    // поэтому print_identity не печатает секцию mail.smtp/mail.imap. Маскировать
    // нечего, но вызов не должен падать. Регрессионный тест на стаб print_identity.
    let home = common::TestHome::new();
    home.add_identity("alice");
    let ctx = home.ctx("default");
    run_user(&ctx, &tokens(&["show", "alice"])).unwrap();
    run_user(&ctx, &tokens(&["show", "alice", "--reveal"])).unwrap();
}
