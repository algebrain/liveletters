use assert_cmd::Command;
use tempfile::TempDir;

fn lltt_cmd() -> Command {
    assert_cmd::Command::cargo_bin("lltt").expect("lltt binary")
}

#[test]
fn user_init_draft_does_not_contain_acct_or_account_id() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("init")
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["user", "init", "alice"])
        .assert()
        .success();

    let draft = home.path().join("drafts").join("alice.toml");
    assert!(draft.exists(), "draft должен быть создан");
    let content = std::fs::read_to_string(&draft).expect("read draft");
    assert!(
        !content.contains("acct_"),
        "draft не должен содержать acct_: {content}"
    );
    assert!(
        !content.contains("account_id"),
        "draft не должен содержать account_id: {content}"
    );
}
