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
account_id = "{name}"
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
}

fn outbox_eml_files(home: &std::path::Path, user: &str) -> Vec<std::path::PathBuf> {
    let store = liveletters_store::Store::open_for_home_dir(home.join("users").join(user)).unwrap();
    let records = store.list_outbox_records().unwrap();
    let mut paths = Vec::new();
    for rec in &records {
        let message = liveletters_protocol::decode_message(&rec.message_body).unwrap();
        let outgoing = liveletters_mime::build_protocol_email(
            "sender@example.org",
            "recipient@example.org",
            &rec.event_type,
            &message,
        )
        .unwrap();
        let path = home.join(format!("{}.eml", rec.event_id));
        std::fs::write(&path, &outgoing.raw_message).unwrap();
        paths.push(path);
    }
    paths
}

fn import_eml(home: &std::path::Path, path: &std::path::Path) {
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["inbox", "import", path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn two_users_subscribe_posts_appear_in_feed() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("init")
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");

    // ── bob подписывается на alice ──
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["sub", "alice@example.org"])
        .assert()
        .success();

    // ── письмо-подписка из bob → alice ──
    let sub_emails = outbox_eml_files(home.path(), "bob");
    assert!(
        !sub_emails.is_empty(),
        "bob should have subscription outbox"
    );
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    for path in &sub_emails {
        import_eml(home.path(), path);
    }

    // ── alice создаёт два поста ──
    for (i, body) in ["Первый пост", "Второй пост"].iter().enumerate() {
        let path = home.path().join(format!("body{i}.txt"));
        std::fs::write(&path, body).unwrap();
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["post", "new", "--body-file", path.to_str().unwrap()])
            .assert()
            .success();
    }

    // ── письма-посты из alice → bob ──
    let post_emails = outbox_eml_files(home.path(), "alice");
    assert!(
        post_emails.len() >= 2,
        "alice should have >=2 outbox records, got {}",
        post_emails.len()
    );
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    for path in &post_emails {
        import_eml(home.path(), path);
    }

    // ── проверка ленты bob ──
    let assert = lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("feed")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(
        stdout.contains("Первый пост"),
        "feed must contain 'Первый пост':\n{stdout}"
    );
    assert!(
        stdout.contains("Второй пост"),
        "feed must contain 'Второй пост':\n{stdout}"
    );
    assert!(
        !stdout.contains("acct_"),
        "acct_* must not appear in feed:\n{stdout}"
    );
    assert!(
        stdout.contains("от alice"),
        "feed must show 'от alice' (nickname):\n{stdout}"
    );
}
