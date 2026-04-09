use liveletters_mail::{
    InMemoryImapMailbox, InMemorySmtpTransport, MailRetryPolicy, TransportError,
    build_protocol_email, decode_protocol_message, extract_liveletters_parts, parse_email,
};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

#[test]
fn protocol_email_round_trip_can_be_sent_fetched_and_decoded() {
    let protocol_message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        "Новая запись в блоге",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            visibility: "public".into(),
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        &protocol_message,
    )
    .expect("raw email should be built");

    let mut smtp = InMemorySmtpTransport::new();
    smtp.send(outgoing.clone()).expect("send should succeed");

    let mut imap = InMemoryImapMailbox::new();
    imap.push_raw_email("message-1", &outgoing.raw_message);

    let fetched = imap.fetch_new().expect("fetch should succeed");
    assert_eq!(fetched.len(), 1);

    let parsed = parse_email(&fetched[0].raw_message).expect("email should parse");
    let extracted = extract_liveletters_parts(&parsed).expect("multipart should extract");
    let decoded = decode_protocol_message(extracted.technical_body()).expect("json should decode");

    assert_eq!(extracted.human_readable_body(), "Новая запись в блоге");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn multipart_fixture_keeps_human_and_technical_parts() {
    let raw_email = include_str!("fixtures/protocol-message.eml");

    let parsed = parse_email(raw_email).expect("fixture should parse");
    let extracted = extract_liveletters_parts(&parsed).expect("fixture parts should extract");
    let decoded = decode_protocol_message(extracted.technical_body()).expect("payload should decode");

    assert_eq!(parsed.subject().as_deref(), Some("LiveLetters fixture"));
    assert_eq!(extracted.human_readable_body(), "Человекочитаемая часть письма.");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::CommentCreated { comment_id, .. } if comment_id == "comment-1"
    ));
}

#[test]
fn auth_error_is_typed_and_not_retried() {
    let policy = MailRetryPolicy::new(3);

    assert!(!policy.should_retry(&TransportError::AuthenticationFailed));
}

#[test]
fn network_error_is_retried_until_limit() {
    let policy = MailRetryPolicy::new(2);

    assert!(policy.should_retry(&TransportError::Network("timeout".into())));
    assert!(policy.allows_attempt(1));
    assert!(policy.allows_attempt(2));
    assert!(!policy.allows_attempt(3));
}
