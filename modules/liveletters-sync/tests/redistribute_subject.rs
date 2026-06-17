//! Тест пересылки комментариев: `event_type` = технический,
//! `subject` = локализованная тема на языке отправителя (A).

mod common;

use common::open_temp_store;
use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage, decode_message,
};
use liveletters_store::{OutboxDelivery, OutboxRecord, SubscriptionRecord};
use liveletters_sync::{SyncEngine, SyncMessageOutcome};

fn identity(nickname: &str, email: &str) -> ProtocolIdentity {
    ProtocolIdentity::new(nickname.to_owned(), email.to_owned()).unwrap()
}

fn protocol_email(
    message_id: &str,
    from: &str,
    to: &str,
    subject: &str,
    message: &ProtocolMessage,
) -> ReceivedEmail {
    let outgoing = build_protocol_email(
        from,
        to,
        subject,
        Some(message.human_readable_body().unwrap_or("")),
        message,
    )
    .unwrap();
    ReceivedEmail {
        message_id: message_id.to_owned(),
        raw_message: outgoing.raw_message,
    }
}

fn outbox_email(from: &str, to: &str, record: &OutboxRecord) -> ReceivedEmail {
    let message = decode_message(&record.message_body).unwrap();
    let outgoing = build_protocol_email(
        from,
        to,
        record.subject.as_deref().unwrap_or("LiveLetters"),
        record.human_readable_body.as_deref(),
        &message,
    )
    .unwrap();
    ReceivedEmail {
        message_id: format!("delivered-{}", record.event_id),
        raw_message: outgoing.raw_message,
    }
}

fn comment_created_email(
    event_id: &str,
    comment_id: &str,
    post_id: &str,
    resource_id: &str,
    author_email: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "comment_created", resource_id, event_id).unwrap(),
        identity(author_email, author_email),
        None,
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
    protocol_email(
        &format!("message-{event_id}"),
        author_email,
        resource_id,
        "Sync fixture",
        &message,
    )
}

fn post_created_email(
    event_id: &str,
    post_id: &str,
    resource_id: &str,
    author_nickname: &str,
    author_email: &str,
) -> ReceivedEmail {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", resource_id, event_id).unwrap(),
        identity(author_nickname, author_email),
        None,
        "Новая запись",
        DomainEventPayload::PostCreated {
            post_id: post_id.into(),
            resource_id: resource_id.into(),
            actor_id: author_email.into(),
            created_at: 1,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    protocol_email(
        &format!("message-{event_id}"),
        author_email,
        resource_id,
        "Sync fixture",
        &message,
    )
}

#[test]
fn redistribute_writes_technical_event_type_and_localized_subject() {
    let (store, _tmp) = open_temp_store();
    // alice (отправитель пересылки) — на английском
    store
        .save_identity(
            "default",
            "alice-publish@example.org",
            "Alice",
            None,
            "en",
            true,
        )
        .unwrap();
    store.save_author("eve@example.org", "Eve", "test").unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("default");

    // eve подписана на alice
    store
        .save_subscription(&SubscriptionRecord {
            resource_email: "alice-publish@example.org".into(),
            subscriber_email: "eve@example.org".into(),
        })
        .unwrap();

    // пост от alice
    let _ = engine
        .ingest_batch(vec![post_created_email(
            "post-1",
            "post-1",
            "alice-publish@example.org",
            "Alice",
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

    let forwarded = liveletters_protocol::decode_message(&redist[0].message_body).unwrap();
    assert_eq!(forwarded.origin().email(), "bob@example.org");
    assert_eq!(
        forwarded.effective_source().email(),
        "alice-publish@example.org"
    );
}

#[test]
fn redistribute_subject_follows_sender_language_ru() {
    let (store, _tmp) = open_temp_store();
    // alice (отправитель пересылки) — на русском
    store
        .save_identity(
            "default",
            "alice-publish@example.org",
            "Алиса",
            None,
            "ru",
            true,
        )
        .unwrap();
    store.save_author("eve@example.org", "Eve", "test").unwrap();
    let engine = SyncEngine::new(&store).with_profile_id("default");

    store
        .save_subscription(&SubscriptionRecord {
            resource_email: "alice-publish@example.org".into(),
            subscriber_email: "eve@example.org".into(),
        })
        .unwrap();

    let _ = engine
        .ingest_batch(vec![post_created_email(
            "post-1",
            "post-1",
            "alice-publish@example.org",
            "Алиса",
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

#[test]
fn redistributed_comment_adds_origin_author_to_recipient_without_subscription_between_them() {
    let (bob_store, _bob_tmp) = open_temp_store();
    bob_store
        .save_identity(
            "default",
            "bob-publish@example.org",
            "Боб",
            None,
            "ru",
            true,
        )
        .unwrap();
    bob_store
        .save_author("alice@example.org", "Алиса", "test")
        .unwrap();
    bob_store
        .save_author("eve@example.org", "Ева", "test")
        .unwrap();
    bob_store
        .save_subscription(&SubscriptionRecord {
            resource_email: "bob-publish@example.org".into(),
            subscriber_email: "alice@example.org".into(),
        })
        .unwrap();
    bob_store
        .save_subscription(&SubscriptionRecord {
            resource_email: "bob-publish@example.org".into(),
            subscriber_email: "eve@example.org".into(),
        })
        .unwrap();
    let bob_engine = SyncEngine::new(&bob_store).with_profile_id("default");

    let bob_post = post_created_email(
        "bob-post-1",
        "bob-post-1",
        "bob-publish@example.org",
        "Боб",
        "bob-publish@example.org",
    );
    bob_engine.ingest_batch(vec![bob_post.clone()]).unwrap();

    let report = bob_engine
        .ingest_batch(vec![comment_created_email(
            "alice-comment-1",
            "alice-comment-1",
            "bob-post-1",
            "bob-publish@example.org",
            "alice@example.org",
        )])
        .unwrap();
    assert!(matches!(
        report.outcomes()[0],
        SyncMessageOutcome::Applied { .. }
    ));

    let outbox = bob_store.list_outbox_records().unwrap();
    let redist = outbox
        .iter()
        .find(|record| record.event_id.starts_with("redistribute:"))
        .expect("Боб должен переслать комментарий Еве");
    let forwarded = decode_message(&redist.message_body).unwrap();
    assert_eq!(forwarded.origin().email(), "alice@example.org");
    assert_eq!(
        forwarded.effective_source().email(),
        "bob-publish@example.org"
    );

    let (eve_store, _eve_tmp) = open_temp_store();
    eve_store
        .save_author("bob-publish@example.org", "Боб", "test")
        .unwrap();
    eve_store
        .save_local_subscriptions("default", &["bob-publish@example.org".to_owned()])
        .unwrap();
    let eve_subscriptions = vec!["bob-publish@example.org".to_owned()];
    let eve_engine =
        SyncEngine::new_with_identity(&eve_store, "eve@example.org", &eve_subscriptions);

    eve_engine.ingest_batch(vec![bob_post]).unwrap();
    eve_engine
        .ingest_batch(vec![outbox_email(
            "bob-publish@example.org",
            "eve@example.org",
            redist,
        )])
        .unwrap();

    let alice = eve_store
        .get_author("alice@example.org")
        .unwrap()
        .expect("Ева должна узнать Алису из origin пересланного комментария");
    assert_eq!(alice.nickname, "alice@example.org");
    assert!(
        eve_store
            .list_subscriptions_for_resource("alice@example.org")
            .unwrap()
            .is_empty()
    );
    assert!(
        eve_store
            .list_subscriptions_for_subscriber("alice@example.org")
            .unwrap()
            .is_empty()
    );
    assert!(
        !eve_store
            .list_local_subscriptions("default")
            .unwrap()
            .contains(&"alice@example.org".to_owned())
    );
}
