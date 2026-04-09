use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};
use liveletters_store::Store;
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

fn protocol_email(
    event_id: &str,
    payload: DomainEventPayload,
    human_body: &str,
) -> ReceivedEmail {
    let (event_type, resource_id) = match &payload {
        DomainEventPayload::PostCreated {
            resource_id, ..
        } => ("post_created", resource_id.as_str()),
        DomainEventPayload::CommentCreated {
            resource_id, ..
        } => ("comment_created", resource_id.as_str()),
        DomainEventPayload::PostHidden {
            resource_id, ..
        } => ("post_hidden", resource_id.as_str()),
        DomainEventPayload::CommentEdited {
            resource_id, ..
        } => ("comment_edited", resource_id.as_str()),
    };

    let protocol_message = ProtocolMessage::new(
        MessageEnvelope::new("1", event_type, resource_id, event_id).unwrap(),
        human_body,
        payload,
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Sync fixture",
        &protocol_message,
    )
    .unwrap();

    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

#[test]
fn valid_post_created_message_is_applied() {
    let store = Store::open_in_memory().unwrap();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-1",
            DomainEventPayload::PostCreated {
                post_id: "post-1".into(),
                resource_id: "blog-1".into(),
                actor_id: "alice".into(),
                created_at: 1,
                visibility: "public".into(),
            },
            "Новая запись",
        )])
        .expect("batch should ingest");

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(report.outcomes()[0], SyncMessageOutcome::Applied { .. }));
    assert_eq!(store.list_posts().unwrap().len(), 1);
    assert_eq!(store.list_raw_event_records().unwrap().len(), 1);
}

#[test]
fn duplicate_event_is_detected_without_reapplying() {
    let store = Store::open_in_memory().unwrap();
    let engine = SyncEngine::new(&store);
    let email_1 = protocol_email(
        "event-1",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1,
            visibility: "public".into(),
        },
        "Новая запись",
    );
    let mut email_2 = email_1.clone();
    email_2.message_id = "message-event-1-duplicate".into();

    engine
        .ingest_batch(vec![email_1, email_2])
        .expect("batch should ingest");

    assert_eq!(store.list_posts().unwrap().len(), 1);
    let raw_messages = store.list_raw_message_records().unwrap();
    assert_eq!(raw_messages.len(), 2);
    assert_eq!(raw_messages[1].status, "duplicate");
}

#[test]
fn malformed_message_is_reported() {
    let store = Store::open_in_memory().unwrap();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![ReceivedEmail {
            message_id: "message-malformed".into(),
            raw_message: "From: broken".into(),
        }])
        .expect("batch should ingest");

    assert!(matches!(report.outcomes()[0], SyncMessageOutcome::Malformed { .. }));
    assert!(store.list_posts().unwrap().is_empty());
    assert_eq!(store.list_raw_message_records().unwrap()[0].status, "malformed");
}

#[test]
fn comment_without_post_is_deferred() {
    let store = Store::open_in_memory().unwrap();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-2",
            DomainEventPayload::CommentCreated {
                comment_id: "comment-1".into(),
                post_id: "missing-post".into(),
                parent_comment_id: None,
                resource_id: "blog-1".into(),
                actor_id: "alice".into(),
                created_at: 2,
                visibility: "public".into(),
            },
            "Новый комментарий",
        )])
        .expect("batch should ingest");

    assert!(matches!(report.outcomes()[0], SyncMessageOutcome::Deferred { .. }));
    assert!(store.list_comments_for_post("missing-post").unwrap().is_empty());
    assert_eq!(store.list_deferred_event_records().unwrap().len(), 1);
}
