use liveletters_mime::{
    build_protocol_email, decode_protocol_message, extract_liveletters_parts, parse_email,
};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

fn sample_message() -> ProtocolMessage {
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

fn sample_protocol_mime() -> String {
    let message = sample_message();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        &message,
    )
    .expect("raw email should be built");

    outgoing.raw_message
}

fn sample_legacy_multipart_mime() -> String {
    let message = ProtocolMessage::new(
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
    .unwrap();

    let technical_payload = liveletters_protocol::encode_message(&message).unwrap();

    format!(
        "From: alice@example.test\nTo: bob@example.test\nSubject: Новая запись\nX-LiveLetters-Protocol: v1\nMIME-Version: 1.0\nContent-Type: multipart/mixed; boundary=\"liveletters-boundary\"\n\n--liveletters-boundary\nContent-Type: text/plain; charset=\"utf-8\"\n\nНовая запись в блоге\n--liveletters-boundary\nContent-Type: application/json\n\n{technical_payload}\n--liveletters-boundary--\n"
    )
}

#[test]
fn parses_headers_and_body_from_protocol_email() {
    let raw = sample_protocol_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    assert_eq!(parsed.subject().as_deref(), Some("Новая запись"));
    assert!(parsed.body().contains("Новая запись в блоге"));
}

#[test]
fn build_protocol_email_uses_inline_text_protocol_block() {
    let raw = sample_protocol_mime();
    assert!(raw.contains("Content-Type: text/plain; charset=\"utf-8\"\n"));
    assert!(raw.contains("X-LiveLetters-Protocol: v1\n"));
    assert!(raw.contains("\n-- \nLiveLetters-Protocol: v1\nLiveLetters-Payload: "));
    assert!(!raw.contains("Content-Type: application/json\n\n"));
    assert!(!raw.contains("multipart/mixed"));
}

#[test]
fn extracts_human_and_technical_parts_from_inline_protocol_email() {
    let raw = sample_protocol_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    assert_eq!(parts.human_readable_body(), "Новая запись в блоге");
    assert!(!parts.technical_body().is_empty());
}

#[test]
fn build_and_decode_round_trip_preserves_payload() {
    let raw = sample_protocol_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    let decoded = decode_protocol_message(parts.technical_body()).expect("payload should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn build_protocol_email_marks_liveletters_protocol_header() {
    let raw = sample_protocol_mime();
    assert!(raw.contains("X-LiveLetters-Protocol: v1\n"));
}

#[test]
fn extracts_human_and_technical_parts_from_legacy_multipart_email() {
    let raw = sample_legacy_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    assert_eq!(parts.human_readable_body(), "Новая запись в блоге");
    let decoded = decode_protocol_message(parts.technical_body()).expect("payload should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn parse_email_rejects_message_without_blank_line_separator() {
    let err = parse_email("Subject: only-headers\nNo body").unwrap_err();
    assert!(matches!(
        err,
        liveletters_mime::MimeError::InvalidEmailFormat(_)
    ));
}

#[test]
fn extract_liveletters_parts_rejects_non_multipart_email() {
    let raw = "Subject: plain\nFrom: a@b\n\nhello there\n";
    let parsed = parse_email(raw).expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();
    assert!(matches!(
        err,
        liveletters_mime::MimeError::InvalidEmailFormat(_)
    ));
}
