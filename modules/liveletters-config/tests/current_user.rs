//! Тесты функций `read_current_identity` и `write_current_identity`.

use std::fs;

use liveletters_config::{
    ConfigError, current_user_path, read_current_identity, write_current_identity,
};
use tempfile::TempDir;

#[test]
fn read_current_identity_returns_no_current_user_when_file_missing() {
    let tmp = TempDir::new().expect("tempdir");
    let err = read_current_identity(tmp.path()).unwrap_err();
    assert!(
        matches!(err, ConfigError::NoCurrentUser(_)),
        "ожидался ConfigError::NoCurrentUser, получили: {err:?}"
    );
    let path = current_user_path(tmp.path());
    assert!(
        !path.exists(),
        "current-user не должен существовать в свежем tempdir"
    );
}

#[test]
fn write_then_read_round_trip() {
    let tmp = TempDir::new().expect("tempdir");
    write_current_identity(tmp.path(), "alice").expect("write current identity");
    let got = read_current_identity(tmp.path()).expect("read current identity");
    assert_eq!(got, "alice");
}

#[test]
fn write_current_identity_overwrites_previous_value() {
    let tmp = TempDir::new().expect("tempdir");
    write_current_identity(tmp.path(), "alice").expect("write alice");
    write_current_identity(tmp.path(), "bob").expect("write bob");
    let got = read_current_identity(tmp.path()).expect("read");
    assert_eq!(got, "bob");
    let raw = fs::read_to_string(current_user_path(tmp.path())).expect("read raw file");
    assert_eq!(
        raw, "bob",
        "файл должен содержать ровно `bob` без лишних пробелов"
    );
}

#[test]
fn read_trims_trailing_newline_added_by_external_editor() {
    let tmp = TempDir::new().expect("tempdir");
    let path = current_user_path(tmp.path());
    fs::write(&path, "alice\n").expect("write with newline");
    let got = read_current_identity(tmp.path()).expect("read");
    assert_eq!(got, "alice", "read должен обрезать \n");
}
