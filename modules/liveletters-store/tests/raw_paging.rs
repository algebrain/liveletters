//! Тесты `Store::list_raw_message_records_paged` (этап 8.20).

use liveletters_store::RawMessageRecord;

mod common;

#[test]
fn paged_returns_newest_first() {
    let (store, _tmp) = common::open_temp_store();

    for i in 1..=3 {
        store
            .save_raw_message_record(&RawMessageRecord {
                message_id: format!("msg-{i}"),
                raw_message: format!("body-{i}"),
                status: "applied".to_owned(),
            })
            .unwrap();
    }

    let page = store.list_raw_message_records_paged(None, 2).unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].message_id, "msg-3");
    assert_eq!(page[1].message_id, "msg-2");
}

#[test]
fn paged_with_status_filter() {
    let (store, _tmp) = common::open_temp_store();

    for (i, status) in [
        ("msg-1", "applied"),
        ("msg-2", "malformed"),
        ("msg-3", "applied"),
    ] {
        store
            .save_raw_message_record(&RawMessageRecord {
                message_id: i.to_owned(),
                raw_message: format!("body-{i}"),
                status: status.to_owned(),
            })
            .unwrap();
    }

    let applied = store
        .list_raw_message_records_paged(Some("applied"), 10)
        .unwrap();
    assert_eq!(applied.len(), 2);
    assert!(applied.iter().all(|r| r.status == "applied"));
    assert_eq!(applied[0].message_id, "msg-3");
    assert_eq!(applied[1].message_id, "msg-1");
}
