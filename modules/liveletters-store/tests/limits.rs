mod common;

use liveletters_store::RawMessageRecord;

use common::open_temp_store;

fn raw(id: &str, status: &str, received_at: u64) -> RawMessageRecord {
    RawMessageRecord {
        message_id: id.into(),
        raw_message: format!("body-{id}"),
        status: status.into(),
        received_at,
    }
}

#[test]
fn count_raw_messages_counts_all() {
    let (store, _tmp) = open_temp_store();
    store
        .save_raw_message_record(&raw("m1", "applied", 0))
        .unwrap();
    store
        .save_raw_message_record(&raw("m2", "malformed", 0))
        .unwrap();
    store
        .save_raw_message_record(&raw("m3", "rate_limited", 0))
        .unwrap();
    assert_eq!(store.count_raw_messages().unwrap(), 3);
    assert_eq!(store.count_raw_messages_by_status("malformed").unwrap(), 1);
    assert_eq!(store.count_raw_messages_by_status("applied").unwrap(), 1);
    assert_eq!(
        store.count_raw_messages_by_status("rate_limited").unwrap(),
        1
    );
}

#[test]
fn cleanup_old_raw_messages_removes_only_old_garbage() {
    let (store, _tmp) = open_temp_store();
    let now = 100_000_000u64;
    let day = 86_400u64;
    // Старый мусор (старше 14 дней) — должен удалиться.
    store
        .save_raw_message_record(&raw("old-mal", "malformed", now - 20 * day))
        .unwrap();
    store
        .save_raw_message_record(&raw("old-inv", "invalid", now - 20 * day))
        .unwrap();
    store
        .save_raw_message_record(&raw("old-rl", "rate_limited", now - 20 * day))
        .unwrap();
    // Свежий мусор — остаётся.
    store
        .save_raw_message_record(&raw("new-mal", "malformed", now))
        .unwrap();
    // Старый applied — не мусор, остаётся.
    store
        .save_raw_message_record(&raw("old-applied", "applied", now - 30 * day))
        .unwrap();

    let deleted = store.cleanup_raw_messages_before(now - 14 * day).unwrap();
    assert_eq!(deleted, 3);
    let remaining: Vec<String> = store
        .list_raw_message_records()
        .unwrap()
        .into_iter()
        .map(|r| r.message_id)
        .collect();
    assert_eq!(remaining.len(), 2);
    assert!(remaining.contains(&"new-mal".to_owned()));
    assert!(remaining.contains(&"old-applied".to_owned()));
}

#[test]
fn enforce_raw_messages_quota_drops_oldest_garbage() {
    let (store, _tmp) = open_temp_store();
    // 5 мусорных, квота 2 → удалить 3 самых старых.
    store
        .save_raw_message_record(&raw("g1", "malformed", 100))
        .unwrap();
    store
        .save_raw_message_record(&raw("g2", "invalid", 200))
        .unwrap();
    store
        .save_raw_message_record(&raw("g3", "rate_limited", 300))
        .unwrap();
    store
        .save_raw_message_record(&raw("g4", "malformed", 400))
        .unwrap();
    store
        .save_raw_message_record(&raw("g5", "invalid", 500))
        .unwrap();
    // applied не считается мусором и не подпадает под квоту.
    store
        .save_raw_message_record(&raw("a1", "applied", 0))
        .unwrap();

    let deleted = store.enforce_raw_messages_quota(2).unwrap();
    assert_eq!(deleted, 3);

    let remaining: Vec<String> = store
        .list_raw_message_records()
        .unwrap()
        .into_iter()
        .map(|r| r.message_id)
        .collect();
    // Остались: 2 самых свежих мусорных (g4, g5) + applied (a1).
    assert!(remaining.contains(&"g4".to_owned()));
    assert!(remaining.contains(&"g5".to_owned()));
    assert!(remaining.contains(&"a1".to_owned()));
    assert!(!remaining.contains(&"g1".to_owned()));
    assert_eq!(remaining.len(), 3);
}

#[test]
fn enforce_raw_messages_quota_noop_when_under_limit() {
    let (store, _tmp) = open_temp_store();
    store
        .save_raw_message_record(&raw("g1", "malformed", 0))
        .unwrap();
    let deleted = store.enforce_raw_messages_quota(10).unwrap();
    assert_eq!(deleted, 0);
    assert_eq!(store.count_raw_messages().unwrap(), 1);
}
