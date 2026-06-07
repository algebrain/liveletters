use liveletters_mail::{
    MailRetryPolicy, TransportError, build_protocol_email, decode_protocol_message,
    extract_liveletters_parts, parse_email,
};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

fn sample_post_created_message() -> ProtocolMessage {
    ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        "Новая запись в блоге",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap()
}

#[test]
fn built_email_can_be_parsed_and_decoded() {
    let message = sample_post_created_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        &message,
    )
    .expect("raw email should be built");

    let parsed = parse_email(&outgoing.raw_message).expect("email should parse");
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
    let decoded =
        decode_protocol_message(extracted.technical_body()).expect("payload should decode");

    assert_eq!(parsed.subject().as_deref(), Some("LiveLetters fixture"));
    assert_eq!(
        extracted.human_readable_body(),
        "Человекочитаемая часть письма."
    );
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

#[test]
fn mime_errors_convert_to_transport_errors() {
    let parsed = parse_email("Subject: plain\n\nhello\n").expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();
    let transport: TransportError = err.into();
    assert!(matches!(transport, TransportError::InvalidEmailFormat(_)));
}

#[test]
fn built_email_round_trips_through_multipart_with_named_json() {
    let message = sample_post_created_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        &message,
    )
    .expect("raw email should be built");

    assert!(
        outgoing
            .raw_message
            .contains("Content-Type: multipart/mixed; boundary=\"liveletters-boundary\"")
    );
    assert!(
        outgoing
            .raw_message
            .contains("Content-Disposition: attachment; filename=\"liveletters.json\"")
    );
    assert!(!outgoing.raw_message.contains("LiveLetters-Payload:"));

    let parsed = parse_email(&outgoing.raw_message).expect("email should parse");
    let extracted = extract_liveletters_parts(&parsed).expect("multipart should extract");
    let decoded = decode_protocol_message(extracted.technical_body()).expect("json should decode");

    assert_eq!(extracted.human_readable_body(), "Новая запись в блоге");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn multipart_filename_appears_in_part_header() {
    let message = sample_post_created_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        &message,
    )
    .expect("raw email should be built");

    let occurrences = outgoing
        .raw_message
        .matches("filename=\"liveletters.json\"")
        .count();
    assert_eq!(
        occurrences, 1,
        "filename должен встречаться ровно один раз в JSON-части"
    );
}

#[test]
fn multipart_email_round_trips_long_cyrillic_body() {
    let long_ru_body = "Привет, мир!\n\nЭто тестовое письмо на русском языке.\n\
                        С новой строки, с цифрами 0123456789 и знаками — «»…\n\
                        — LiveLetters";
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        long_ru_body,
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись от alice в blog-1",
        &message,
    )
    .expect("raw email should be built");

    let parsed = parse_email(&outgoing.raw_message).expect("email should parse");
    let extracted = extract_liveletters_parts(&parsed).expect("multipart should extract");
    let decoded = decode_protocol_message(extracted.technical_body()).expect("json should decode");

    assert_eq!(extracted.human_readable_body(), long_ru_body);
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[cfg(feature = "network")]
mod network_flow;
