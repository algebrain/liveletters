use assert_cmd::Command;
use tempfile::TempDir;

fn lltt_cmd() -> Command {
    assert_cmd::Command::cargo_bin("lltt").expect("lltt binary")
}

fn write_identity(home: &std::path::Path, name: &str) {
    std::fs::create_dir_all(home.join("identities")).expect("identities dir");
    std::fs::write(
        home.join("identities").join(format!("{name}.toml")),
        format!(
            r#"
display_name = "{name}"
[mail]
publish = "{name}@example.org"
receive = ["{name}@example.org"]
[meta]
resources_owned = ["{name}@example.org"]
"#
        ),
    )
    .expect("write identity");
}

fn setup_user(home: &std::path::Path, name: &str) {
    write_identity(home, name);
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["cu", name])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "language", "ru"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "nickname", name])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "email_address", &format!("{name}@example.org")])
        .assert()
        .success();
}

#[test]
fn local_post_and_comment_thread_does_not_show_acct_prefix() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("init")
        .assert()
        .success();
    setup_user(home.path(), "alice");

    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Пост Алисы").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let post_id =
        liveletters_store::Store::open_for_home_dir(home.path().join("users").join("alice"))
            .expect("open user store")
            .list_posts()
            .expect("list posts")[0]
            .post_id
            .clone();

    let comment_body = home.path().join("c.txt");
    std::fs::write(&comment_body, "Мой комментарий").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args([
            "comment",
            "new",
            &post_id,
            "--body-file",
            comment_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    let assert = lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["thread", &post_id])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(
        !stdout.contains("acct_"),
        "thread не должен показывать acct_<name> для локально созданных постов и комментариев:\n{stdout}"
    );
}
