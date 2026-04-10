use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;

use liveletters_mail::{
    ConfiguredImapMailbox, ConfiguredSmtpTransport, FetchStatus, ImapMailboxConfig,
    InMemoryImapMailbox, InMemorySmtpTransport, MailAuth, MailRetryPolicy, MailSecurity,
    MailboxCursor,
    SendStatus, SmtpTransportConfig, TransportError, build_protocol_email,
    decode_protocol_message, extract_liveletters_parts, parse_email,
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
    let send_status = smtp.send(outgoing.clone()).expect("send should succeed");
    assert_eq!(send_status, SendStatus::Sent);

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

#[test]
fn configured_smtp_transport_sends_message_over_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let address = listener.local_addr().expect("address should exist");
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("client should connect");
        socket
            .write_all(b"220 localhost ESMTP ready\r\n")
            .expect("greeting should be written");

        let mut reader = BufReader::new(socket.try_clone().expect("socket should clone"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("EHLO should be read");
        assert!(line.starts_with("EHLO local.test"));
        socket
            .write_all(b"250-localhost\r\n250 AUTH PLAIN\r\n")
            .expect("EHLO response should be written");

        line.clear();
        reader.read_line(&mut line).expect("AUTH should be read");
        assert!(line.starts_with("AUTH PLAIN "));
        socket
            .write_all(b"235 2.7.0 Authentication successful\r\n")
            .expect("AUTH response should be written");

        line.clear();
        reader.read_line(&mut line).expect("MAIL FROM should be read");
        assert!(line.starts_with("MAIL FROM:<alice@example.test>"));
        socket
            .write_all(b"250 2.1.0 Ok\r\n")
            .expect("MAIL FROM response should be written");

        line.clear();
        reader.read_line(&mut line).expect("RCPT TO should be read");
        assert!(line.starts_with("RCPT TO:<bob@example.test>"));
        socket
            .write_all(b"250 2.1.5 Ok\r\n")
            .expect("RCPT response should be written");

        line.clear();
        reader.read_line(&mut line).expect("DATA should be read");
        assert_eq!(line, "DATA\r\n");
        socket
            .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
            .expect("DATA response should be written");

        let mut data = Vec::new();
        loop {
            let mut byte = [0_u8; 1];
            reader.read_exact(&mut byte).expect("message byte should be read");
            data.push(byte[0]);
            if data.ends_with(b"\r\n.\r\n") {
                break;
            }
        }
        let raw_message = String::from_utf8(data).expect("SMTP data should be UTF-8");
        assert!(raw_message.contains("Subject: Новая запись\r\n"));
        assert!(raw_message.contains("Content-Type: multipart/mixed; boundary=\"liveletters-boundary\""));
        socket
            .write_all(b"250 2.0.0 Queued\r\n")
            .expect("queue response should be written");

        line.clear();
        reader.read_line(&mut line).expect("QUIT should be read");
        assert_eq!(line, "QUIT\r\n");
        socket
            .write_all(b"221 2.0.0 Bye\r\n")
            .expect("QUIT response should be written");
    });

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

    let transport = ConfiguredSmtpTransport::new(SmtpTransportConfig::new(
        "127.0.0.1",
        address.port(),
        "local.test",
        MailSecurity::None,
        MailAuth::Password {
            username: "alice".into(),
            password: "secret".into(),
        },
    ));

    let status = transport.send(&outgoing).expect("real SMTP send should succeed");
    assert_eq!(status, SendStatus::Sent);

    server.join().expect("SMTP server thread should finish");
}

#[test]
fn configured_imap_mailbox_fetches_messages_with_cursor() {
    let protocol_message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-9").unwrap(),
        "Живое письмо",
        DomainEventPayload::PostCreated {
            post_id: "post-9".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_123,
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "IMAP письмо",
        &protocol_message,
    )
    .expect("raw email should be built");
    let raw_message = outgoing.raw_message.clone();
    let literal_size = raw_message.len();

    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let address = listener.local_addr().expect("address should exist");
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("client should connect");
        socket
            .write_all(b"* OK IMAP4rev1 ready\r\n")
            .expect("greeting should be written");
        let mut reader = BufReader::new(socket.try_clone().expect("socket should clone"));
        let mut line = String::new();

        reader.read_line(&mut line).expect("LOGIN should be read");
        assert!(line.starts_with("a001 LOGIN "));
        socket
            .write_all(b"a001 OK LOGIN completed\r\n")
            .expect("LOGIN response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SELECT should be read");
        assert_eq!(line, "a002 SELECT INBOX\r\n");
        socket
            .write_all(b"* 1 EXISTS\r\na002 OK [READ-WRITE] SELECT completed\r\n")
            .expect("SELECT response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SEARCH should be read");
        assert_eq!(line, "a003 UID SEARCH UID 11:*\r\n");
        socket
            .write_all(b"* SEARCH 11\r\na003 OK SEARCH completed\r\n")
            .expect("SEARCH response should be written");

        line.clear();
        reader.read_line(&mut line).expect("FETCH should be read");
        assert_eq!(line, "a004 UID FETCH 11 BODY.PEEK[]\r\n");
        let fetch_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{literal_size}}})\r\n{raw_message}\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(fetch_response.as_bytes())
            .expect("FETCH response should be written");

        line.clear();
        reader.read_line(&mut line).expect("LOGOUT should be read");
        assert_eq!(line, "a005 LOGOUT\r\n");
        socket
            .write_all(b"* BYE Logging out\r\na005 OK LOGOUT completed\r\n")
            .expect("LOGOUT response should be written");
    });

    let mailbox = ConfiguredImapMailbox::new(ImapMailboxConfig::new(
        "127.0.0.1",
        address.port(),
        "INBOX",
        MailSecurity::None,
        MailAuth::Password {
            username: "alice".into(),
            password: "secret".into(),
        },
    ));
    let batch = mailbox
        .fetch_new(&MailboxCursor::from_last_seen_uid(10))
        .expect("real IMAP fetch should succeed");

    assert_eq!(batch.status(), &FetchStatus::Fetched { message_count: 1 });
    assert_eq!(batch.emails().len(), 1);
    assert_eq!(batch.next_cursor().last_seen_uid(), Some(11));
    assert!(batch.emails()[0].raw_message.contains("Subject: IMAP письмо"));

    server.join().expect("IMAP server thread should finish");
}
