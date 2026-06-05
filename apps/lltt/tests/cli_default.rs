//! Регрессионные тесты default-ветки: `LIVELETTERS_HOME` не задан,
//! `lltt` использует `<HOME>/.liveletters/` (Unix) или `<USERPROFILE%gt;/.liveletters/` (Windows).
//!
//! Чтобы не трогать реальный `~/.liveletters` пользователя, тест пробрасывает
//! `HOME` (и `USERPROFILE`) в **дочерний процесс** через `Command::env`, не трогая
//! окружение самого теста. Это важно: мутация глобального `HOME` ломала бы
//! параллельные тесты в `cli_*` файлах.

use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::TempDir;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

fn lltt_with_fake_user_home(tmp: &TempDir) -> Command {
    let mut cmd = lltt();
    cmd.env_remove("LIVELETTERS_HOME")
        .env("HOME", tmp.path())
        .env("USERPROFILE", tmp.path());
    cmd
}

#[test]
fn init_without_liveletters_home_creates_dot_liveletters_under_user_home() {
    let tmp = TempDir::new().expect("tempdir");
    lltt_with_fake_user_home(&tmp)
        .arg("init")
        .assert()
        .success();
    let home = tmp.path().join(".liveletters");
    assert!(home.join("liveletters.sqlite3").exists(), "БД не создана");
    assert!(
        home.join("mail-password-obfuscation.key").exists(),
        "ключ не создан"
    );
    assert!(
        home.join("identities").is_dir(),
        "каталог identities не создан"
    );
    assert!(home.join("drafts").is_dir(), "каталог drafts не создан");
    assert!(!home.join("identities/default.toml").exists());
    assert!(!home.join("current-user").exists());
}

#[test]
fn user_list_without_liveletters_home_reads_default_dot_liveletters() {
    let tmp = TempDir::new().expect("tempdir");
    // Сначала инициализируем default-каталог.
    lltt_with_fake_user_home(&tmp)
        .arg("init")
        .assert()
        .success();
    // Затем проверяем, что `user list` работает без выбранного пользователя.
    lltt_with_fake_user_home(&tmp)
        .arg("user")
        .arg("list")
        .assert()
        .success();
}
