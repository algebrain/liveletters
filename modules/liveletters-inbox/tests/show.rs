//! Тесты `liveletters-inbox::show` (этап 8.23).

use liveletters_inbox::{InboxError, ShowArgs, show};
use liveletters_store::RawMessageRecord;

mod common;

#[test]
fn show_returns_full_body() {
    let (store, tmp) = common::open_temp_store();

    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "msg-1".to_owned(),
            raw_message: "From: a\nSubject: hello\n\nbody content".to_owned(),
            status: "applied".to_owned(),
        })
        .unwrap();

    let args = ShowArgs {
        id: "msg-1".to_owned(),
    };
    show::run(tmp.path(), &args).expect("show должен найти msg-1");
}

#[test]
fn show_unknown_id_returns_not_found() {
    let (_store, tmp) = common::open_temp_store();

    let args = ShowArgs {
        id: "missing".to_owned(),
    };
    let err = show::run(tmp.path(), &args).unwrap_err();
    assert!(matches!(err, InboxError::MessageNotFound(id) if id == "missing"));
}

#[test]
fn show_empty_id_returns_not_found() {
    let (_store, tmp) = common::open_temp_store();

    let args = ShowArgs { id: String::new() };
    let err = show::run(tmp.path(), &args).unwrap_err();
    assert!(matches!(err, InboxError::MessageNotFound(id) if id.is_empty()));
}
