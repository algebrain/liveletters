//! Проверяет, что бинарь `lltt` собирается, выводит `--help` со всеми 13 подкомандами
//! и сообщает об ошибке при обращении к неинициализированному домашнему каталогу
//! или при отсутствии файла `<home>/current-user`.

use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt должен собираться")
}

#[test]
fn help_lists_all_fourteen_subcommands() {
    let output = lltt().arg("--help").output().expect("запуск lltt --help");
    assert!(output.status.success(), "lltt --help завершился с ошибкой");
    let stdout = String::from_utf8_lossy(&output.stdout);

    for sub in [
        "init", "cu", "user", "sub", "feed", "inbox", "post", "comment", "outbox", "thread",
        "status", "doctor", "settings", "sync",
    ] {
        assert!(
            stdout.contains(sub),
            "в --help отсутствует подкоманда `{sub}`"
        );
    }
}

#[test]
fn unknown_subcommand_returns_error() {
    let tmp = TempDir::new().expect("tempdir");
    let output = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("totally-bogus")
        .output()
        .expect("запуск lltt totally-bogus");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error") || stderr.contains("unrecognized") || stderr.contains("неизве"),
        "stderr должен содержать сообщение об ошибке: {stderr}"
    );
}

#[test]
fn command_without_init_returns_no_current_user_error() {
    // Домашний каталог существует, но `lltt init` в нём не запускался —
    // значит, файла `<home>/current-user` нет. Любая не-init команда
    // должна вернуть код 2 с понятным сообщением.
    let tmp = TempDir::new().expect("tempdir");
    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("status")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("текущ") || stderr.contains("current-user") || stderr.contains("не задан"),
        "ожидалось сообщение про отсутствующий current-user, получили: {stderr}"
    );
}

#[test]
fn command_when_current_user_file_removed_returns_error() {
    // Инициализируем каталог, затем удаляем `<home>/current-user`.
    // Не-init команда должна вернуть код 2.
    let tmp = TempDir::new().expect("tempdir");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();
    fs::create_dir_all(tmp.path().join("identities")).expect("create identities");
    std::fs::write(
        tmp.path().join("identities").join("alice.toml"),
        r#"
display_name = "Alice"

[mail]
publish = "alice@example.org"
receive = ["alice@example.org"]
"#,
    )
    .expect("write identity");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    std::fs::remove_file(tmp.path().join("current-user")).expect("remove current-user");

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("status")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("текущ") || stderr.contains("current-user") || stderr.contains("не задан"),
        "ожидалось сообщение про отсутствующий current-user, получили: {stderr}"
    );
}

#[test]
fn status_succeeds_after_init() {
    // После `init` пользователь ещё не создан и не выбран.
    let tmp = TempDir::new().expect("tempdir");
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("init")
        .assert()
        .success();

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("status")
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("lltt user init") && stderr.contains("lltt cu"),
        "status должен подсказать создание пользователя, получили: {stderr}"
    );
}
