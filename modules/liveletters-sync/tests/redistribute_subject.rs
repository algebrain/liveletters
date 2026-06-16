//! Тест пересылки комментариев: `event_type` = технический,
//! `subject` = локализованная тема на языке отправителя (A).

mod common;

use common::open_temp_store;
use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};
use liveletters_store::{OutboxDelivery, SubscriptionRecord, UserSettingsRecord};
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

fn comment_created_email(
    event_id: &str,
    comment_id: &str,
    post_id: &str,
    resource_id: &str,
    author_email: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_created", resource_id, event_id).unwrap(),
        "Новый комментарий",
        DomainEventPayload::CommentCreated {
            comment_id: comment_id.into(),
            post_id: post_id.into(),
            parent_comment_id: None,
            resource_id: resource_id.into(),
            actor_id: author_email.into(),
            created_at: 1,
            body: "Текст комментария".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        author_email,
        resource_id,
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();
    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

fn post_created_email(event_id: &str, post_id: &str, resource_id: &str) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", resource_id, event_id).unwrap(),
        "Новая запись",
        DomainEventPayload::PostCreated {
            post_id: post_id.into(),
            resource_id: resource_id.into(),
            actor_id: "alice-publish@example.org".into(),
            created_at: 1,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice-publish@example.org",
        resource_id,
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();
    ReceivedEmail {
        message_id: format!("message-{event_id}"),
        raw_message: outgoing.raw_message,
    }
}

#[test]
fn redistribute_writes_technical_event_type_and_localized_subject() {
    let (store, _tmp) = open_temp_store();
    // alice (отправитель пересылки) — на английском
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "default".into(),
            nickname: "Alice".into(),
            email_address: "alice-publish@example.org".into(),
            avatar_url: None,
            language: "en".into(),
            setup_completed: true,
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("default");

    // eve подписана на alice
    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "eve@example.org".into(),
        })
        .unwrap();

    // пост от alice
    let _ = engine
        .ingest_batch(vec![post_created_email(
            "post-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    // bob комментирует пост
    let report = engine
        .ingest_batch(vec![comment_created_email(
            "comment-1",
            "comment-1",
            "post-1",
            "alice-publish@example.org",
            "bob@example.org",
        )])
        .unwrap();
    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));

    let outbox = store.list_outbox_records().unwrap();
    let redist: Vec<_> = outbox
        .iter()
        .filter(|r| r.event_id.starts_with("redistribute:"))
        .collect();
    assert_eq!(redist.len(), 1, "ожидалась ровно 1 запись пересылки");

    // event_type — технический идентификатор, не локализованная строка
    assert_eq!(
        redist[0].event_type, "comment_created",
        "event_type пересылки должен быть техническим, получили {:?}",
        redist[0].event_type
    );
    match &redist[0].delivery {
        OutboxDelivery::Direct(addrs) => {
            assert_eq!(addrs, &vec!["eve@example.org".to_owned()]);
        }
        other => panic!("ожидался Direct, получили {other:?}"),
    }
    // subject — локализован на языке отправителя (alice = en)
    let subject = redist[0]
        .subject
        .as_deref()
        .expect("redistribute должен иметь локализованный subject");
    assert!(
        subject.contains("New comment in"),
        "subject должен быть на английском (язык alice), получили: {subject:?}"
    );
}

#[test]
fn redistribute_subject_follows_sender_language_ru() {
    let (store, _tmp) = open_temp_store();
    // alice (отправитель) — на русском
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "default".into(),
            nickname: "Алиса".into(),
            email_address: "alice-publish@example.org".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("default");

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "alice-publish@example.org".into(),
            subscriber_delivery_address: "eve@example.org".into(),
        })
        .unwrap();

    let _ = engine
        .ingest_batch(vec![post_created_email(
            "post-1",
            "post-1",
            "alice-publish@example.org",
        )])
        .unwrap();

    engine
        .ingest_batch(vec![comment_created_email(
            "comment-1",
            "comment-1",
            "post-1",
            "alice-publish@example.org",
            "bob@example.org",
        )])
        .unwrap();

    let outbox = store.list_outbox_records().unwrap();
    let redist: Vec<_> = outbox
        .iter()
        .filter(|r| r.event_id.starts_with("redistribute:"))
        .collect();
    assert_eq!(redist.len(), 1);
    let subject = redist[0]
        .subject
        .as_deref()
        .expect("redistribute должен иметь subject");
    assert!(
        subject.contains("Новый комментарий в"),
        "subject должен быть на русском, получили: {subject:?}"
    );
}
