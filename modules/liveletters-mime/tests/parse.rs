use liveletters_mime::{
    MimeError, MimeLimits, build_protocol_email, decode_protocol_message,
    extract_liveletters_parts, extract_liveletters_parts_with_limits, parse_email,
    parse_email_with_limits,
};
use liveletters_protocol::{
    DomainEventPayload, MessageEnvelope, ProtocolIdentity, ProtocolMessage,
};

fn alice() -> ProtocolIdentity {
    ProtocolIdentity::new("Alice", "alice@example.test").unwrap()
}

fn sample_message() -> ProtocolMessage {
    ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        alice(),
        None,
        "Новая запись в блоге",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            created_at: 1_710_000_000,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap()
}

fn sample_multipart_mime() -> String {
    let message = sample_message();

    let technical_payload = liveletters_protocol::encode_message(&message).unwrap();

    format!(
        "From: alice@example.test\nTo: bob@example.test\nSubject: Новая запись\nX-LiveLetters-Protocol: v1\nMIME-Version: 1.0\nContent-Type: multipart/mixed; boundary=\"liveletters-boundary\"\n\n--liveletters-boundary\nContent-Type: text/plain; charset=\"utf-8\"\n\nНовая запись в блоге\n--liveletters-boundary\nContent-Type: application/json; name=\"liveletters.json\"\nContent-Disposition: attachment; filename=\"liveletters.json\"\n\n{technical_payload}\n--liveletters-boundary--\n"
    )
}

#[test]
fn parses_headers_and_body_from_protocol_email() {
    let raw = sample_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    assert_eq!(parsed.subject().as_deref(), Some("Новая запись"));
    assert!(parsed.body().contains("Новая запись в блоге"));
}

#[test]
fn build_protocol_email_uses_multipart_with_named_json_attachment() {
    let message = sample_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    let raw = outgoing.raw_message;

    assert!(raw.contains("Content-Type: multipart/mixed; boundary=\"liveletters-boundary\""));
    assert!(raw.contains("MIME-Version: 1.0"));
    assert!(raw.contains("Content-Disposition: attachment; filename=\"liveletters.json\""));
    assert!(raw.contains("Content-Type: text/plain; charset=\"utf-8\""));
    assert!(raw.contains("Content-Type: application/json"));
    assert!(!raw.contains("LiveLetters-Payload:"));
    assert!(!raw.contains("base64"));
}

#[test]
fn parses_human_and_protocol_from_multipart_with_filename() {
    let raw = sample_multipart_mime();
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
fn protocol_part_filename_is_liveletters_json() {
    let raw = sample_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    assert!(raw.contains("Content-Disposition: attachment; filename=\"liveletters.json\""));
    assert!(!parts.technical_body().is_empty());
}

#[test]
fn build_and_decode_round_trip_preserves_payload() {
    let message = sample_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    let parsed = parse_email(&outgoing.raw_message).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    let decoded = decode_protocol_message(parts.technical_body()).expect("payload should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn build_protocol_email_marks_liveletters_protocol_header() {
    let message = sample_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    assert!(
        outgoing
            .raw_message
            .contains("X-LiveLetters-Protocol: v1\n")
    );
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

#[test]
fn multipart_email_preserves_long_cyrillic_human_body() {
    let long_ru_body = "Привет, мир!\n\nЭто тестовое письмо на русском языке.\n\
                        С новой строки, с цифрами 0123456789 и знаками — «»…\n\
                        — LiveLetters";
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-1").unwrap(),
        alice(),
        None,
        long_ru_body,
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
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
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    let parsed = parse_email(&outgoing.raw_message).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    assert_eq!(parts.human_readable_body(), long_ru_body);
    let decoded = decode_protocol_message(parts.technical_body()).expect("payload should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn parse_email_handles_folded_headers_without_crashing() {
    let raw = include_str!("fixtures/folded-headers.eml");
    let parsed = parse_email(raw).expect("email with folded headers should parse");
    assert_eq!(parsed.subject(), None);
    assert!(parsed.body().contains("Hello Bob"));
}

#[test]
fn extract_parts_from_email_with_folded_headers() {
    let raw = include_str!("fixtures/folded-headers.eml");
    let parsed = parse_email(raw).expect("email should parse");
    let parts = extract_liveletters_parts(&parsed).expect("parts should extract");
    assert_eq!(
        parts.human_readable_body(),
        "Hello Bob, this message tests folded headers."
    );
    let decoded = decode_protocol_message(parts.technical_body()).expect("payload should decode");
    assert!(matches!(
        decoded.payload(),
        DomainEventPayload::PostCreated { post_id, .. } if post_id == "post-1"
    ));
}

#[test]
fn build_protocol_email_encodes_cyrillic_subject_with_rfc2047() {
    let message = sample_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись от alice",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    assert!(
        !outgoing
            .raw_message
            .contains("Subject: Новая запись от alice\n"),
        "raw Cyrillic should NOT appear in Subject header (must be RFC 2047-encoded)"
    );
    assert!(
        outgoing.raw_message.contains("Subject: =?utf-8?B?"),
        "Subject should be RFC 2047-encoded with base64"
    );
}

#[test]
fn build_protocol_email_keeps_ascii_subject_unchanged() {
    let message = sample_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "New post",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    assert!(
        outgoing.raw_message.contains("Subject: New post\n"),
        "ASCII subject should appear as-is"
    );
}

#[test]
fn encoded_subject_round_trips_through_parse() {
    let message = sample_message();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email should be built");
    let parsed = parse_email(&outgoing.raw_message).expect("should parse");
    assert_eq!(parsed.subject().as_deref(), Some("Новая запись"));
}

fn multipart_with_parts(parts: &[&str]) -> String {
    format!(
        "From: alice@example.test\n\
         To: bob@example.test\n\
         Subject: MIME security\n\
         X-LiveLetters-Protocol: v1\n\
         MIME-Version: 1.0\n\
         Content-Type: multipart/mixed; boundary=\"liveletters-boundary\"\n\
         \n\
         {}\n\
         --liveletters-boundary--\n",
        parts.join("")
    )
}

fn text_part(body: &str) -> String {
    format!(
        "--liveletters-boundary\n\
         Content-Type: text/plain; charset=\"utf-8\"\n\
         \n\
         {body}\n"
    )
}

fn json_part(filename: Option<&str>, body: &str) -> String {
    match filename {
        Some(filename) => format!(
            "--liveletters-boundary\n\
             Content-Type: application/json; name=\"{filename}\"\n\
             Content-Disposition: attachment; filename=\"{filename}\"\n\
             \n\
             {body}\n"
        ),
        None => format!(
            "--liveletters-boundary\n\
             Content-Type: application/json\n\
             \n\
             {body}\n"
        ),
    }
}

fn binary_part() -> String {
    "--liveletters-boundary\n\
     Content-Type: application/octet-stream\n\
     Content-Disposition: attachment; filename=\"extra.bin\"\n\
     \n\
     bytes\n"
        .to_owned()
}

fn encoded_sample_message(event_id: &str) -> String {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", event_id).unwrap(),
        alice(),
        None,
        "Новая запись в блоге",
        DomainEventPayload::PostCreated {
            post_id: format!("post-{event_id}"),
            resource_id: "blog-1".into(),
            created_at: 1_710_000_000,
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    liveletters_protocol::encode_message(&message).unwrap()
}

#[test]
fn extract_rejects_duplicate_liveletters_json_parts() {
    let first = encoded_sample_message("first-json");
    let second = encoded_sample_message("second-json");
    let raw = multipart_with_parts(&[
        &text_part("Новая запись"),
        &json_part(Some("liveletters.json"), &first),
        &json_part(Some("liveletters.json"), &second),
    ]);

    let parsed = parse_email(&raw).expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_json_part_without_liveletters_filename() {
    let raw = multipart_with_parts(&[
        &text_part("Новая запись"),
        &json_part(Some("payload.json"), &encoded_sample_message("wrong-name")),
    ]);

    let parsed = parse_email(&raw).expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_json_part_without_filename() {
    let raw = multipart_with_parts(&[
        &text_part("Новая запись"),
        &json_part(None, &encoded_sample_message("missing-name")),
    ]);

    let parsed = parse_email(&raw).expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_extra_attachment_without_manifest() {
    let raw = multipart_with_parts(&[
        &text_part("Новая запись"),
        &json_part(
            Some("liveletters.json"),
            &encoded_sample_message("extra-attachment"),
        ),
        &binary_part(),
    ]);

    let parsed = parse_email(&raw).expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_duplicate_text_plain_parts() {
    let raw = multipart_with_parts(&[
        &text_part("Первая часть"),
        &text_part("Вторая часть"),
        &json_part(
            Some("liveletters.json"),
            &encoded_sample_message("duplicate-text"),
        ),
    ]);

    let parsed = parse_email(&raw).expect("email should parse");
    let err = extract_liveletters_parts(&parsed).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn parse_email_rejects_raw_message_over_limit() {
    let raw = sample_multipart_mime();
    let limits = MimeLimits {
        max_raw_email_bytes: raw.len() - 1,
        ..MimeLimits::default()
    };

    let err = parse_email_with_limits(&raw, limits).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_json_over_limit() {
    let raw = sample_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let limits = MimeLimits {
        max_json_bytes: 32,
        ..MimeLimits::default()
    };

    let err = extract_liveletters_parts_with_limits(&parsed, limits).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_human_body_over_limit() {
    let raw = sample_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let limits = MimeLimits {
        max_human_bytes: 8,
        ..MimeLimits::default()
    };

    let err = extract_liveletters_parts_with_limits(&parsed, limits).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_too_many_mime_parts() {
    let raw = sample_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let limits = MimeLimits {
        max_parts: 2,
        ..MimeLimits::default()
    };

    let err = extract_liveletters_parts_with_limits(&parsed, limits).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}

#[test]
fn extract_rejects_mime_tree_deeper_than_limit() {
    let raw = sample_multipart_mime();
    let parsed = parse_email(&raw).expect("email should parse");
    let limits = MimeLimits {
        max_depth: 0,
        ..MimeLimits::default()
    };

    let err = extract_liveletters_parts_with_limits(&parsed, limits).unwrap_err();

    assert!(matches!(err, MimeError::InvalidEmailFormat(_)));
}
