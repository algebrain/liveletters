//! Тесты `liveletters_inbox::import::run`: импорт `.eml`-файлов в БД через `SyncEngine`.

mod common;

use std::fs;
use std::path::Path;

use liveletters_inbox::import;
use liveletters_store::Store;
use tempfile::TempDir;

fn open_store(home: &Path) -> Store {
    Store::open_for_home_dir(home).expect("store opens in temp home")
}

fn init_home_with_store(home: &Path) {
    let _ = open_store(home);
}

#[test]
fn import_eml_with_valid_post_inserts_row() {
    let tmp = TempDir::new().unwrap();
    init_home_with_store(tmp.path());

    let eml = common::write_valid_post_eml(tmp.path(), "Привет, это первый пост");
    import::run(tmp.path(), std::slice::from_ref(&eml)).unwrap();

    let store = open_store(tmp.path());
    let posts = store.list_posts().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].post_id, "post-1");
}

#[test]
fn import_twice_yields_duplicate() {
    let tmp = TempDir::new().unwrap();
    init_home_with_store(tmp.path());

    let eml = common::write_valid_post_eml(tmp.path(), "тело");
    import::run(tmp.path(), std::slice::from_ref(&eml)).unwrap();
    import::run(tmp.path(), std::slice::from_ref(&eml)).unwrap();

    let store = open_store(tmp.path());
    let posts = store.list_posts().unwrap();
    assert_eq!(posts.len(), 1);
}

#[test]
fn import_missing_file_returns_err() {
    let tmp = TempDir::new().unwrap();
    init_home_with_store(tmp.path());

    let missing = tmp.path().join("does-not-exist.eml");
    let err = import::run(tmp.path(), std::slice::from_ref(&missing)).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("не найден"), "got: {msg}");
}

#[test]
fn import_malformed_eml_returns_err() {
    let tmp = TempDir::new().unwrap();
    init_home_with_store(tmp.path());

    let eml = tmp.path().join("bad.eml");
    fs::write(&eml, "Subject: only-headers\nNo body").unwrap();
    let err = import::run(tmp.path(), std::slice::from_ref(&eml)).unwrap_err();
    let _ = err;
}
