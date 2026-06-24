mod common;

use liveletters_mail::{ReceivedEmail, build_protocol_email};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage,
};
use liveletters_sync::{IngestLimits, SyncEngine, SyncMessageOutcome};

use common::open_temp_store;

fn identity(nickname: &str, email: &str) -> ProtocolIdentity {
    ProtocolIdentity::new(nickname.to_owned(), email.to_owned()).unwrap()
}

#[derive(Clone)]
struct Input<'a> {
    smtp_from: &'a str,
    smtp_to: &'a str,
    event_id: &'a str,
    origin: ProtocolIdentity,
    source: Option<ProtocolIdentity>,
    payload: DomainEventPayload,
}

fn protocol_email(input: Input<'_>) -> ReceivedEmail {
    let (event_type, resource_id) = match &input.payload {
        DomainEventPayload::PostCreated { resource_id, .. } => {
            ("post_created", resource_id.as_str())
        }
        DomainEventPayload::CommentCreated { resource_id, .. } => {
            ("comment_created", resource_id.as_str())
        }
        DomainEventPayload::SubscriptionRequested { resource_id, .. } => {
            ("subscription_requested", resource_id.as_str())
        }
        _ => ("post_created", "x@example.test"),
    };
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", event_type, resource_id, input.event_id).unwrap(),
        input.origin,
        input.source,
        "limits fixture",
        input.payload,
    )
    .unwrap();
    let outgoing = build_protocol_email(
        input.smtp_from,
        input.smtp_to,
        "Limits fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap();
    ReceivedEmail {
        message_id: format!("message-{}", input.event_id),
        raw_message: outgoing.raw_message,
    }
}

fn missing_post_comment(event_id: &str, origin_email: &str) -> ReceivedEmail {
    protocol_email(Input {
        smtp_from: origin_email,
        smtp_to: "bob@example.test",
        event_id,
        origin: identity("Mallory", origin_email),
        source: Some(identity("Bob", "bob@example.test")),
        payload: DomainEventPayload::CommentCreated {
            comment_id: format!("comment-{event_id}"),
            post_id: "missing-post".into(),
            parent_comment_id: None,
            resource_id: "bob@example.test".into(),
            created_at: 1_710_000_000,
            body: "Комментарий к отсутствующему посту".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    })
}

fn post_created_by(event_id: &str, author_email: &str) -> ReceivedEmail {
    protocol_email(Input {
        smtp_from: author_email,
        smtp_to: author_email,
        event_id,
        origin: identity("Author", author_email),
        source: None,
        payload: DomainEventPayload::PostCreated {
            post_id: format!("post-{event_id}"),
            resource_id: author_email.to_owned(),
            created_at: 1_710_000_000,
            body: "Пост автора".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    })
}

fn count(outcomes: &[SyncMessageOutcome], pred: impl Fn(&SyncMessageOutcome) -> bool) -> usize {
    outcomes.iter().filter(|o| pred(o)).count()
}

#[test]
fn deferred_total_is_capped() {
    let (store, _tmp) = open_temp_store();
    let limits = IngestLimits {
        max_deferred_total: 10,
        ..IngestLimits::disabled()
    };
    let engine = SyncEngine::new_with_limits(&store, limits);

    let messages: Vec<_> = (0..50)
        .map(|i| missing_post_comment(&format!("ev-{i}"), "mallory@example.test"))
        .collect();
    let report = engine.ingest_batch(messages).expect("batch");

    let deferred = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::Deferred { .. })
    });
    let rate_limited = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::RateLimited { .. })
    });
    assert_eq!(deferred, 10);
    assert_eq!(rate_limited, 40);
    assert_eq!(store.count_deferred_events().unwrap(), 10);
}

#[test]
fn deferred_per_origin_is_capped() {
    let (store, _tmp) = open_temp_store();
    let limits = IngestLimits {
        max_deferred_total: 100,
        max_deferred_per_origin: 2,
        ..IngestLimits::disabled()
    };
    let engine = SyncEngine::new_with_limits(&store, limits);

    let origins = ["m1@example.test", "m2@example.test", "m3@example.test"];
    let mut messages = Vec::new();
    let mut id = 0;
    for origin in origins {
        for _ in 0..3 {
            messages.push(missing_post_comment(&format!("ev-{id}"), origin));
            id += 1;
        }
    }
    let report = engine.ingest_batch(messages).expect("batch");

    let deferred = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::Deferred { .. })
    });
    let rate_limited = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::RateLimited { .. })
    });
    // 3 origin × 2 разрешённых = 6 Deferred; 3 origin × 1 запрещённый = 3 RateLimited.
    assert_eq!(deferred, 6);
    assert_eq!(rate_limited, 3);
}

#[test]
fn new_authors_are_capped() {
    let (store, _tmp) = open_temp_store();
    let limits = IngestLimits {
        max_new_authors_per_batch: 20,
        ..IngestLimits::disabled()
    };
    let engine = SyncEngine::new_with_limits(&store, limits);

    let messages: Vec<_> = (0..200)
        .map(|i| post_created_by(&format!("ev-{i}"), &format!("u{i}@example.test")))
        .collect();
    let report = engine.ingest_batch(messages).expect("batch");

    let applied = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::Applied { .. })
    });
    let rate_limited = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::RateLimited { .. })
    });
    assert_eq!(applied, 20);
    assert_eq!(rate_limited, 180);
    assert_eq!(store.list_author_emails().unwrap().len(), 20);
}

#[test]
fn auto_responses_are_capped() {
    let (store, _tmp) = open_temp_store();
    store
        .save_identity("bob", "bob@example.test", "Bob", None, "ru", true)
        .unwrap();
    let limits = IngestLimits {
        max_auto_responses_per_batch: 5,
        ..IngestLimits::disabled()
    };
    let engine = SyncEngine::new_with_limits(&store, limits).with_profile_id("bob");

    let messages: Vec<_> = (0..30)
        .map(|i| {
            let victim = format!("victim{i}@example.test");
            protocol_email(Input {
                smtp_from: "attacker@example.test",
                smtp_to: "bob@example.test",
                event_id: &format!("sub-{i}"),
                origin: identity("Victim", &victim),
                source: None,
                payload: DomainEventPayload::SubscriptionRequested {
                    resource_id: "bob@example.test".into(),
                    subscriber_delivery_address: victim,
                    created_at: 1_710_000_400,
                },
            })
        })
        .collect();
    let report = engine.ingest_batch(messages).expect("batch");

    let outbox = store.list_outbox_records().unwrap();
    let auto_responses = outbox
        .iter()
        .filter(|r| r.event_type == "subscription_confirmed")
        .count();
    assert!(auto_responses <= 5, "auto_responses={auto_responses}");
    // Подписка как локальное состояние владельца сохраняется для всех запросов.
    let subs = store
        .list_subscriptions_for_resource("bob@example.test")
        .unwrap();
    assert_eq!(subs.len(), 30);
    // Ни одного исходящего письма жертве сверх квоты.
    let victims_in_outbox = outbox
        .iter()
        .filter(|r| {
            r.event_type == "subscription_confirmed"
                && r.delivery
                    == liveletters_store::OutboxDelivery::Direct(vec![
                        "victim5@example.test".into(),
                    ])
        })
        .count();
    assert_eq!(victims_in_outbox, 0);
    let _ = report;
}

#[test]
fn events_per_origin_is_capped() {
    let (store, _tmp) = open_temp_store();
    let limits = IngestLimits {
        max_events_per_origin: 10,
        ..IngestLimits::disabled()
    };
    let engine = SyncEngine::new_with_limits(&store, limits);

    let messages: Vec<_> = (0..100)
        .map(|i| post_created_by(&format!("ev-{i}"), "same@example.test"))
        .collect();
    let report = engine.ingest_batch(messages).expect("batch");

    let applied = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::Applied { .. })
    });
    let rate_limited = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::RateLimited { .. })
    });
    assert_eq!(applied, 10);
    assert_eq!(rate_limited, 90);
}

#[test]
fn normal_flow_is_not_rate_limited() {
    let (store, _tmp) = open_temp_store();
    let engine = SyncEngine::new_with_limits(&store, IngestLimits::default());

    let messages = vec![
        post_created_by("p1", "alice@example.test"),
        post_created_by("p2", "bob@example.test"),
        post_created_by("p3", "carol@example.test"),
    ];
    let report = engine.ingest_batch(messages).expect("batch");

    let rate_limited = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::RateLimited { .. })
    });
    assert_eq!(rate_limited, 0);
    let applied = count(report.outcomes(), |o| {
        matches!(o, SyncMessageOutcome::Applied { .. })
    });
    assert_eq!(applied, 3);
}
