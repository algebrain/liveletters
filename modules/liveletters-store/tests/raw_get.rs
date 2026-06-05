//! Тесты `Store::get_raw_message_record` (этап 8.22).

use liveletters_store::RawMessageRecord;

mod common;

#[test]
fn get_returns_existing() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "msg-1".to_owned(),
            raw_message: "raw-body".to_owned(),
            status: "applied".to_owned(),
        })
        .unwrap();

    let got = store.get_raw_message_record("msg-1").unwrap().unwrap();
    assert_eq!(got.message_id, "msg-1");
    assert_eq!(got.raw_message, "raw-body");
    assert_eq!(got.status, "applied");
}

#[test]
fn get_returns_none_for_missing() {
    let (store, _tmp) = common::open_temp_store();

    let got = store.get_raw_message_record("missing").unwrap();
    assert!(got.is_none());
}
