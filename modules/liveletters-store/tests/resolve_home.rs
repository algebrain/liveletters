//! Тесты разрешения домашнего каталога `lltt` без мутации глобального окружения.
//!
//! Тесты используют чистую функцию [`resolve_data_dir`] и передают ей
//! `EnvOverrides` напрямую, поэтому `cargo test` не вызывает `std::env::set_var`
//! и не трогает переменные окружения родительского процесса.

use liveletters_store::{EnvOverrides, resolve_data_dir};

fn temp_path(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("lltt-{tag}-{}-{}", std::process::id(), nanos))
}

#[test]
fn resolve_data_dir_prefers_liveletters_home() {
    let explicit = temp_path("explicit");
    let got = resolve_data_dir(&EnvOverrides {
        liveletters_home: Some(explicit.clone()),
        ..Default::default()
    });
    assert_eq!(got, Some(explicit.clone()));
    let _ = std::fs::remove_dir_all(&explicit);
}

#[test]
fn resolve_data_dir_falls_back_to_home_with_suffix() {
    let user = temp_path("user");
    let got = resolve_data_dir(&EnvOverrides {
        home: Some(user.clone()),
        ..Default::default()
    });
    assert_eq!(got, Some(user.join(".liveletters")));
    let _ = std::fs::remove_dir_all(&user);
}

#[test]
fn resolve_data_dir_falls_back_to_userprofile_when_home_missing() {
    let user = temp_path("profile");
    let got = resolve_data_dir(&EnvOverrides {
        home: None,
        userprofile: Some(user.clone()),
        ..Default::default()
    });
    assert_eq!(got, Some(user.join(".liveletters")));
    let _ = std::fs::remove_dir_all(&user);
}

#[test]
fn resolve_data_dir_prefers_home_over_userprofile() {
    let unix_user = temp_path("unix");
    let win_user = temp_path("win");
    let got = resolve_data_dir(&EnvOverrides {
        home: Some(unix_user.clone()),
        userprofile: Some(win_user.clone()),
        ..Default::default()
    });
    assert_eq!(got, Some(unix_user.join(".liveletters")));
    let _ = std::fs::remove_dir_all(&unix_user);
    let _ = std::fs::remove_dir_all(&win_user);
}

#[test]
fn resolve_data_dir_returns_none_without_any_env() {
    let got = resolve_data_dir(&EnvOverrides::default());
    assert_eq!(got, None);
}
