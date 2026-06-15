use liveletters_store::{OutboxDelivery, OutboxRecord, Store};
use tempfile::TempDir;

fn open_store() -> (TempDir, Store) {
    let tmp = TempDir::new().expect("tempdir");
    let store = Store::open_for_home_dir(tmp.path()).expect("store opens");
    (tmp, store)
}

#[test]
fn delete_removes_record() {
    let (_tmp, store) = open_store();
    let record = OutboxRecord {
        event_id: "ev-1".into(),
        event_type: "post_created".into(),
        resource_id: "blog-1".into(),
        delivery: OutboxDelivery::ResourceSubscribers,
        message_body: "{}".into(),
        message_id: None,
        subject: None,
    };
    store.save_outbox_record(&record).expect("save");

    let deleted = store.delete_outbox_record("ev-1").expect("delete");
    assert!(deleted);

    let remaining = store.list_outbox_records().expect("list");
    assert!(
        remaining.is_empty(),
        "outbox must be empty, got {remaining:?}"
    );
}

#[test]
fn delete_returns_false_for_missing() {
    let (_tmp, store) = open_store();
    let deleted = store
        .delete_outbox_record("missing")
        .expect("delete missing");
    assert!(!deleted);
}
