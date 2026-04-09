use liveletters_diagnostics::{DiagnosticsReader, HealthStatus};
use liveletters_store::{
    DeferredEventRecord, OutboxRecord, RawMessageRecord, Store,
};

#[test]
fn builds_sync_health_from_store_state() {
    let store = Store::open_in_memory().unwrap();
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-1".into(),
            raw_message: "From: alice@example.test\n\nok".into(),
            status: "applied".into(),
        })
        .unwrap();
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-2".into(),
            raw_message: "From: alice@example.test\n\nbad".into(),
            status: "malformed".into(),
        })
        .unwrap();
    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "event-2".into(),
            event_type: "comment_created".into(),
            reason: "missing_post".into(),
            payload_json: "{\"kind\":\"comment_created\"}".into(),
        })
        .unwrap();

    let diagnostics = DiagnosticsReader::new(&store)
        .build_snapshot()
        .unwrap();

    assert_eq!(diagnostics.sync_health().applied_messages, 1);
    assert_eq!(diagnostics.sync_health().malformed_messages, 1);
    assert_eq!(diagnostics.sync_health().deferred_events, 1);
    assert_eq!(diagnostics.sync_health().status, HealthStatus::Degraded);
}

#[test]
fn raw_message_preview_masks_email_addresses() {
    let store = Store::open_in_memory().unwrap();
    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-1".into(),
            raw_message: "From: alice@example.test\nTo: bob@example.test\n\nhello".into(),
            status: "applied".into(),
        })
        .unwrap();

    let diagnostics = DiagnosticsReader::new(&store)
        .build_snapshot()
        .unwrap();

    let preview = &diagnostics.raw_messages()[0].preview;
    assert!(!preview.contains("alice@example.test"));
    assert!(!preview.contains("bob@example.test"));
    assert!(preview.contains("***@example.test"));
}

#[test]
fn outbox_entries_are_exposed_through_stable_dto() {
    let store = Store::open_in_memory().unwrap();
    store
        .save_outbox_record(&OutboxRecord {
            event_id: "event-1".into(),
            event_type: "post_created".into(),
            resource_id: "blog-1".into(),
            message_body: "{\"kind\":\"post_created\",\"actor\":\"alice@example.test\"}".into(),
        })
        .unwrap();

    let diagnostics = DiagnosticsReader::new(&store)
        .build_snapshot()
        .unwrap();

    assert_eq!(diagnostics.outbox_entries().len(), 1);
    assert_eq!(diagnostics.outbox_entries()[0].event_type, "post_created");
    assert!(diagnostics.outbox_entries()[0]
        .preview
        .contains("***@example.test"));
}
