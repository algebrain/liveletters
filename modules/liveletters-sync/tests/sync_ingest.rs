mod common;

use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage,
};
use liveletters_store::{CommentRecord, PostRecord};
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

use common::open_temp_store;

fn ensure_author(store: &liveletters_store::Store, email: &str, nickname: &str) {
    store.save_author(email, nickname, "test").unwrap();
}

fn identity(nickname: &str, email: &str) -> ProtocolIdentity {
    ProtocolIdentity::new(nickname.to_owned(), email.to_owned()).unwrap()
}

fn protocol_email(event_id: &str, payload: DomainEventPayload, human_body: &str) -> ReceivedEmail {
    let (event_type, resource_id) = match &payload {
        DomainEventPayload::PostCreated { resource_id, .. } => {
            ("post_created", resource_id.as_str())
        }
        DomainEventPayload::CommentCreated { resource_id, .. } => {
            ("comment_created", resource_id.as_str())
        }
        DomainEventPayload::PostHidden { resource_id, .. } => ("post_hidden", resource_id.as_str()),
        DomainEventPayload::CommentEdited { resource_id, .. } => {
            ("comment_edited", resource_id.as_str())
        }
        DomainEventPayload::SubscriptionRequested { resource_id, .. } => {
            ("subscription_requested", resource_id.as_str())
        }
        DomainEventPayload::SubscriptionConfirmed { resource_id, .. } => {
            ("subscription_confirmed", resource_id.as_str())
        }
        DomainEventPayload::SubscriptionRevoked { resource_id, .. } => {
            ("subscription_revoked", resource_id.as_str())
        }
        DomainEventPayload::FriendAdded { resource_id, .. } => {
            ("friend_added", resource_id.as_str())
        }
    };

    let protocol_message = ProtocolMessage::new(
        MessageEnvelope::new("1", event_type, resource_id, event_id).unwrap(),
        identity("Alice", "alice@example.test"),
        None,
        human_body,
        payload,
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Sync fixture",
        Some(protocol_message.human_readable_body().unwrap_or("")),
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
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "blog-1", "blog");
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-1",
            DomainEventPayload::PostCreated {
                post_id: "post-1".into(),
                resource_id: "blog-1".into(),
                created_at: 1,
                body: "Текст поста".into(),
                body_format: "plain".into(),
                visibility: "public".into(),
            },
            "alice написал:\n\nТекст поста",
        )])
        .expect("batch should ingest");

    assert_eq!(report.outcomes().len(), 1);
    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    let posts = store.list_posts().unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].body, "Текст поста");
    assert_eq!(store.list_raw_event_records().unwrap().len(), 1);
}

#[test]
fn duplicate_event_is_detected_without_reapplying() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "blog-1", "blog");
    let engine = SyncEngine::new(&store);
    let email_1 = protocol_email(
        "event-1",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            created_at: 1,
            body: "Текст поста".into(),
            body_format: "plain".into(),
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
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![ReceivedEmail {
            message_id: "message-malformed".into(),
            raw_message: "From: broken".into(),
        }])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Malformed { .. }
    ));
    assert!(store.list_posts().unwrap().is_empty());
    assert_eq!(
        store.list_raw_message_records().unwrap()[0].status,
        "malformed"
    );
}

#[test]
fn comment_without_post_is_deferred() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-2",
            DomainEventPayload::CommentCreated {
                comment_id: "comment-1".into(),
                post_id: "missing-post".into(),
                parent_comment_id: None,
                resource_id: "blog-1".into(),
                created_at: 2,
                body: "Новый комментарий".into(),
                body_format: "plain".into(),
                visibility: "public".into(),
            },
            "Новый комментарий",
        )])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Deferred { .. }
    ));
    assert!(
        store
            .list_comments_for_post("missing-post")
            .unwrap()
            .is_empty()
    );
    assert_eq!(store.list_deferred_event_records().unwrap().len(), 1);
}

#[test]
fn replayed_post_created_is_reported_separately_from_duplicate_event_id() {
    let (store, _tmp) = open_temp_store();
    // Создаём автора для FK.
    store
        .save_author("blog-1", "blog", "self")
        .expect("save resource author");
    store
        .save_author("alice", "alice", "self")
        .expect("save author");
    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice".into(),
            created_at: 1,
            body: "Existing post".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-replay-1",
            DomainEventPayload::PostCreated {
                post_id: "post-1".into(),
                resource_id: "blog-1".into(),
                created_at: 1,
                body: "Старая запись".into(),
                body_format: "plain".into(),
                visibility: "public".into(),
            },
            "Старая запись",
        )])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Replay { .. }
    ));
    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events.len(), 1);
    assert_eq!(raw_events[0].apply_status, "replay");
}

#[test]
fn unauthorized_comment_edit_is_rejected() {
    let (store, _tmp) = open_temp_store();
    // Создаём автора для FK.
    store
        .save_author("blog-1", "blog", "self")
        .expect("save resource author");
    store
        .save_author("alice", "alice", "self")
        .expect("save author");
    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice".into(),
            created_at: 1,
            body: "Post".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    store
        .save_comment_record(&CommentRecord {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: None,
            author_email: "alice".into(),
            created_at: 2,
            body: "Original".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-unauthorized-1",
            DomainEventPayload::CommentEdited {
                comment_id: "comment-1".into(),
                post_id: "post-1".into(),
                resource_id: "blog-1".into(),
                created_at: 3,
                body: "Hacked".into(),
                visibility: "public".into(),
            },
            "Незаконное редактирование",
        )])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Unauthorized { .. }
    ));
    assert_eq!(
        store.get_comment_record("comment-1").unwrap().unwrap().body,
        "Original"
    );
    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events[0].apply_status, "unauthorized");
}

#[test]
fn invalid_event_with_mismatched_resource_id_is_rejected() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let protocol_message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-envelope", "event-invalid-1").unwrap(),
        identity("Alice", "alice@example.test"),
        None,
        "Некорректное событие",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-payload".into(),
            created_at: 1,
            body: "Некорректное событие".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Invalid event",
        Some(protocol_message.human_readable_body().unwrap_or("")),
        &protocol_message,
    )
    .unwrap();

    let report = engine
        .ingest_batch(vec![ReceivedEmail {
            message_id: "message-invalid-1".into(),
            raw_message: outgoing.raw_message,
        }])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Invalid { .. }
    ));
    assert!(store.list_posts().unwrap().is_empty());
    let raw_events = store.list_raw_event_records().unwrap();
    assert_eq!(raw_events[0].apply_status, "invalid");
}

#[test]
fn invalid_post_created_with_blank_body_is_rejected() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-blank-post",
            DomainEventPayload::PostCreated {
                post_id: "post-blank".into(),
                resource_id: "blog-1".into(),
                created_at: 1,
                body: "   ".into(),
                body_format: "plain".into(),
                visibility: "public".into(),
            },
            "Пустая запись",
        )])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Invalid { .. }
    ));
    assert!(store.list_posts().unwrap().is_empty());
}

#[test]
fn invalid_post_created_with_unknown_body_format_is_rejected() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new(&store);

    let report = engine
        .ingest_batch(vec![protocol_email(
            "event-unknown-format",
            DomainEventPayload::PostCreated {
                post_id: "post-format".into(),
                resource_id: "blog-1".into(),
                created_at: 1,
                body: "Текст".into(),
                body_format: "rst".into(),
                visibility: "public".into(),
            },
            "Неизвестный формат",
        )])
        .expect("batch should ingest");

    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Invalid { .. }
    ));
    assert!(store.list_posts().unwrap().is_empty());
}

#[test]
fn deferred_events_can_be_reprocessed_after_dependencies_appear() {
    let (store, _tmp) = open_temp_store();
    ensure_author(&store, "blog-1", "blog");
    let engine = SyncEngine::new(&store);

    engine
        .ingest_batch(vec![protocol_email(
            "event-comment-1",
            DomainEventPayload::CommentCreated {
                comment_id: "comment-1".into(),
                post_id: "post-1".into(),
                parent_comment_id: None,
                resource_id: "blog-1".into(),
                created_at: 2,
                body: "Комментарий раньше поста".into(),
                body_format: "plain".into(),
                visibility: "public".into(),
            },
            "Комментарий раньше поста",
        )])
        .expect("initial batch should ingest");

    engine
        .ingest_batch(vec![protocol_email(
            "event-post-1",
            DomainEventPayload::PostCreated {
                post_id: "post-1".into(),
                resource_id: "blog-1".into(),
                created_at: 1,
                body: "Пост появился позже".into(),
                body_format: "plain".into(),
                visibility: "public".into(),
            },
            "Пост появился позже",
        )])
        .expect("post batch should ingest");

    let replay_report = engine
        .reprocess_deferred()
        .expect("deferred events should reprocess");

    assert_eq!(replay_report.outcomes().len(), 1);
    assert!(matches!(
        replay_report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));
    assert_eq!(store.list_comments_for_post("post-1").unwrap().len(), 1);
    assert!(store.list_deferred_event_records().unwrap().is_empty());
}
