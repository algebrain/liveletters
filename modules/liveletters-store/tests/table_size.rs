//! Тесты `Store::table_size` (этап 8.25).

use liveletters_store::{PostRecord, StoreError};

mod common;

#[test]
fn table_size_zero_on_empty() {
    let (store, _tmp) = common::open_temp_store();

    // SQLite резервирует корневую страницу (4 KiB) даже для пустых
    // таблиц; «0 байт» невозможен. Сравниваем с порогом 4 KiB.
    let posts = store.table_size("posts").unwrap();
    let comments = store.table_size("comments").unwrap();
    let raw = store.table_size("raw_messages").unwrap();
    assert!(posts <= 4096, "posts = {posts}");
    assert!(comments <= 4096, "comments = {comments}");
    assert!(raw <= 4096, "raw = {raw}");
}

#[test]
fn table_size_grows_after_insert() {
    let (store, _tmp) = common::open_temp_store();
    common::ensure_author(&store, "blog-1", "blog");
    common::ensure_author(&store, "alice", "alice");

    let before = store.table_size("posts").unwrap();
    for i in 0..50 {
        store
            .save_post_record(&PostRecord {
                post_id: format!("post-{i}"),
                resource_email: "blog-1".to_owned(),
                author_email: "alice".to_owned(),
                created_at: 1_710_000_000 + i,
                body: format!("Тело записи номер {i} {}", "x".repeat(200)),
                visibility: "public".to_owned(),
                hidden: false,
            })
            .unwrap();
    }

    let after = store.table_size("posts").unwrap();
    assert!(after > before, "before = {before}, after = {after}");
}

#[test]
fn table_size_rejects_unknown_table() {
    let (store, _tmp) = common::open_temp_store();

    let err = store.table_size("evil'; DROP TABLE posts;--").unwrap_err();
    assert!(matches!(err, StoreError::InvalidTable(_)), "err = {err:?}");
}
