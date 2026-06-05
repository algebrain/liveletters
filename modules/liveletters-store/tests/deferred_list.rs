//! Тесты `Store::list_deferred_events(limit)` (этап 8.25).

use liveletters_store::DeferredEventRecord;

mod common;

#[test]
fn list_deferred_events_returns_latest_first() {
    let (store, _tmp) = common::open_temp_store();

    for i in 1..=3 {
        store
            .save_deferred_event_record(&DeferredEventRecord {
                event_id: format!("event-{i}"),
                event_type: "post_created".to_owned(),
                reason: format!("reason-{i}"),
                payload_json: "{}".to_owned(),
            })
            .unwrap();
    }

    let latest = store.list_deferred_events(1).unwrap();
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].event_id, "event-3");

    let all = store.list_deferred_events(10).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].event_id, "event-3");
    assert_eq!(all[2].event_id, "event-1");
}
