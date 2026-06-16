//! Интеграционные тесты команды `lltt answer` (синоним `lltt comment new`).

use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn create_post(tmp: &TempDir) -> String {
    let body_path = tmp.path().join("body.txt");
    fs::write(&body_path, "Запись").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let db =
        rusqlite::Connection::open(tmp.path().join("users/alice/liveletters.sqlite3")).unwrap();
    db.query_row("SELECT post_id FROM posts LIMIT 1", [], |r| {
        r.get::<_, String>(0)
    })
    .unwrap()
}

fn open_db(tmp: &TempDir) -> rusqlite::Connection {
    rusqlite::Connection::open(tmp.path().join("users/alice/liveletters.sqlite3")).unwrap()
}

#[test]
fn answer_creates_top_level_comment_on_post_by_prefix() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    let body_path = tmp.path().join("c.txt");
    fs::write(&body_path, "Первый комментарий").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "answer",
            &post_id,
            "--body-file",
            body_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("комментарий создан:"));

    let db = open_db(&tmp);
    let (body, parent): (String, Option<String>) = db
        .query_row(
            "SELECT body, parent_comment_id FROM comments LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(body, "Первый комментарий");
    assert_eq!(
        parent, None,
        "top-level: parent_comment_id должен быть NULL"
    );
}

#[test]
fn answer_replies_to_comment_by_prefix() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    let root_body = tmp.path().join("root.txt");
    fs::write(&root_body, "Корневой").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "answer",
            &post_id,
            "--body-file",
            root_body.to_str().unwrap(),
        ])
        .assert()
        .success();
    let db = open_db(&tmp);
    let root_id: String = db
        .query_row("SELECT comment_id FROM comments LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();

    let reply_body = tmp.path().join("reply.txt");
    fs::write(&reply_body, "Ответ").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "answer",
            &root_id,
            "--body-file",
            reply_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    let db = open_db(&tmp);
    let (body, parent, post_id_in_db): (String, Option<String>, String) = db
        .query_row(
            "SELECT body, parent_comment_id, post_id FROM comments WHERE body = 'Ответ'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(body, "Ответ");
    assert_eq!(parent.as_deref(), Some(root_id.as_str()));
    assert_eq!(
        post_id_in_db, post_id,
        "post_id подтягивается из parent-комментария"
    );
}

#[test]
fn answer_rejects_unknown_post_with_human_message() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let body_path = tmp.path().join("c.txt");
    fs::write(&body_path, "текст").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "answer",
            "post-doesnotexist000",
            "--body-file",
            body_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("не найден"));
}

#[test]
fn answer_rejects_invalid_prefix() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let body_path = tmp.path().join("c.txt");
    fs::write(&body_path, "текст").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "answer",
            "foo-bar",
            "--body-file",
            body_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("post-"))
        .stderr(contains("comment-"));
}

#[test]
fn comment_new_with_positional_target_works_as_synonym() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    let body_path = tmp.path().join("c.txt");
    fs::write(&body_path, "Синоним").unwrap();

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args([
            "comment",
            "new",
            &post_id,
            "--body-file",
            body_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("комментарий создан:"));
}

#[test]
fn answer_reads_body_from_stdin_via_pipe() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");
    let post_id = create_post(&tmp);

    // Никаких --body-file, никаких файлов: тело приходит из stdin,
    // как в `echo "..." | lltt answer ...`.
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("answer")
        .arg(&post_id)
        .write_stdin("this is my answer\n")
        .assert()
        .success()
        .stdout(contains("комментарий создан:"));

    let db = open_db(&tmp);
    let body: String = db
        .query_row("SELECT body FROM comments LIMIT 1", [], |r| r.get(0))
        .unwrap();
    // Хвостовой перевод строки от `echo` срезается при сохранении
    // (body тримится в `CommentBody::new`, см. values.rs:12).
    assert_eq!(body, "this is my answer");
}
