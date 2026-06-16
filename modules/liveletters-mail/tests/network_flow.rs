#![cfg(feature = "network")]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;

use liveletters_mail::{
    ConfiguredImapMailbox, ConfiguredSmtpTransport, FetchStatus, ImapMailboxConfig, MailAuth,
    MailSecurity, MailboxCursor, SendStatus, SmtpTransportConfig, build_protocol_email,
};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

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
        reader
            .read_line(&mut line)
            .expect("MAIL FROM should be read");
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
            reader
                .read_exact(&mut byte)
                .expect("message byte should be read");
            data.push(byte[0]);
            if data.ends_with(b"\r\n.\r\n") {
                break;
            }
        }
        let raw_message = String::from_utf8(data).expect("SMTP data should be UTF-8");
        assert!(raw_message.contains("Subject: =?utf-8?B?"));
        assert!(raw_message.contains("\r\n"));
        assert!(
            raw_message
                .contains("Content-Type: multipart/mixed; boundary=\"liveletters-boundary\"")
        );
        assert!(raw_message.contains("filename=\"liveletters.json\""));
        assert!(!raw_message.contains("LiveLetters-Payload: "));
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
            body: "Текст поста".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Новая запись",
        Some(protocol_message.human_readable_body().unwrap_or("")),
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

    let status = transport
        .send(&outgoing)
        .expect("real SMTP send should succeed");
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
            body: "Живое письмо".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "IMAP письмо",
        Some(protocol_message.human_readable_body().unwrap_or("")),
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
        assert_eq!(
            line,
            "a003 UID SEARCH UID 11:* HEADER X-LiveLetters-Protocol v1\r\n"
        );
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
    assert!(
        batch.emails()[0]
            .raw_message
            .contains("Subject: =?utf-8?B?")
    );

    server.join().expect("IMAP server thread should finish");
}

#[test]
fn configured_imap_mailbox_falls_back_to_header_fetch_when_search_header_is_unsupported() {
    let raw_message =
        "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\nhello\r\n".to_owned();
    let literal_size = raw_message.len();
    let ordinary_header = "Subject: Ordinary\r\n\r\n";
    let ordinary_header_size = ordinary_header.len();
    let liveletters_header = "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\n";
    let liveletters_header_size = liveletters_header.len();
    let expected_raw_message = raw_message.clone();

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
        socket
            .write_all(b"a001 OK LOGIN completed\r\n")
            .expect("LOGIN response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SELECT should be read");
        socket
            .write_all(b"* 2 EXISTS\r\na002 OK [READ-WRITE] SELECT completed\r\n")
            .expect("SELECT response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH HEADER should be read");
        assert_eq!(
            line,
            "a003 UID SEARCH UID 1:* HEADER X-LiveLetters-Protocol v1\r\n"
        );
        socket
            .write_all(b"a003 BAD unsupported search key\r\n")
            .expect("SEARCH HEADER response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH all should be read");
        assert_eq!(line, "a003 UID SEARCH UID 1:*\r\n");
        socket
            .write_all(b"* SEARCH 10 11\r\na003 OK SEARCH completed\r\n")
            .expect("SEARCH all response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("ordinary HEADER FETCH should be read");
        assert_eq!(
            line,
            "a004 UID FETCH 10 BODY.PEEK[HEADER.FIELDS (X-LiveLetters-Protocol)]\r\n"
        );
        let ordinary_response = format!(
            "* 1 FETCH (UID 10 BODY[] {{{ordinary_header_size}}})\r\n{ordinary_header})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(ordinary_response.as_bytes())
            .expect("ordinary HEADER FETCH response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("LiveLetters HEADER FETCH should be read");
        assert_eq!(
            line,
            "a004 UID FETCH 11 BODY.PEEK[HEADER.FIELDS (X-LiveLetters-Protocol)]\r\n"
        );
        let liveletters_header_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{liveletters_header_size}}})\r\n{liveletters_header})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(liveletters_header_response.as_bytes())
            .expect("LiveLetters HEADER FETCH response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("LiveLetters body FETCH should be read");
        assert_eq!(line, "a004 UID FETCH 11 BODY.PEEK[]\r\n");
        let fetch_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{literal_size}}})\r\n{raw_message})\r\na004 OK FETCH completed\r\n"
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
        .fetch_new(&MailboxCursor::from_last_seen_uid(0))
        .expect("real IMAP fetch should succeed");

    assert_eq!(batch.emails().len(), 1);
    assert_eq!(batch.emails()[0].message_id, "imap-uid-11");
    assert_eq!(batch.emails()[0].raw_message, expected_raw_message);
    assert_eq!(batch.next_cursor().last_seen_uid(), Some(11));

    server.join().expect("IMAP server thread should finish");
}

#[test]
fn configured_imap_mailbox_falls_back_to_header_fetch_when_primary_returns_zero_results() {
    let raw_message =
        "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\nhello\r\n".to_owned();
    let literal_size = raw_message.len();
    let liveletters_header = "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\n";
    let liveletters_header_size = liveletters_header.len();
    let expected_raw_message = raw_message.clone();

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
        socket
            .write_all(b"a001 OK LOGIN completed\r\n")
            .expect("LOGIN response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SELECT should be read");
        socket
            .write_all(b"* 1 EXISTS\r\na002 OK [READ-WRITE] SELECT completed\r\n")
            .expect("SELECT response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH HEADER should be read");
        assert_eq!(
            line,
            "a003 UID SEARCH UID 1:* HEADER X-LiveLetters-Protocol v1\r\n"
        );
        socket
            .write_all(b"* SEARCH\r\na003 OK UID SEARCH Completed.\r\n")
            .expect("SEARCH HEADER response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH all should be read");
        assert_eq!(line, "a003 UID SEARCH UID 1:*\r\n");
        socket
            .write_all(b"* SEARCH 11\r\na003 OK SEARCH completed\r\n")
            .expect("SEARCH all response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("HEADER FETCH should be read");
        assert_eq!(
            line,
            "a004 UID FETCH 11 BODY.PEEK[HEADER.FIELDS (X-LiveLetters-Protocol)]\r\n"
        );
        let header_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{liveletters_header_size}}})\r\n{liveletters_header})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(header_response.as_bytes())
            .expect("HEADER FETCH response should be written");

        line.clear();
        reader
            .read_line(&mut line)
            .expect("body FETCH should be read");
        assert_eq!(line, "a004 UID FETCH 11 BODY.PEEK[]\r\n");
        let fetch_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{literal_size}}})\r\n{raw_message})\r\na004 OK FETCH completed\r\n"
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
        .fetch_new(&MailboxCursor::from_last_seen_uid(0))
        .expect("fetch should succeed via fallback path");

    assert_eq!(batch.emails().len(), 1);
    assert_eq!(batch.emails()[0].message_id, "imap-uid-11");
    assert_eq!(batch.emails()[0].raw_message, expected_raw_message);
    assert_eq!(batch.next_cursor().last_seen_uid(), Some(11));

    server.join().expect("IMAP server thread should finish");
}

#[test]
fn configured_imap_mailbox_preserves_fetch_literal_bytes() {
    let raw_message =
        "Subject: IMAP письмо\r\nFrom: alice@example.test\r\n\r\nПривет\r\nмир\r\n".to_owned();
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
        socket
            .write_all(b"a001 OK LOGIN completed\r\n")
            .expect("LOGIN response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SELECT should be read");
        socket
            .write_all(b"* 1 EXISTS\r\na002 OK [READ-WRITE] SELECT completed\r\n")
            .expect("SELECT response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SEARCH should be read");
        socket
            .write_all(b"* SEARCH 11\r\na003 OK SEARCH completed\r\n")
            .expect("SEARCH response should be written");

        line.clear();
        reader.read_line(&mut line).expect("FETCH should be read");
        let fetch_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{literal_size}}})\r\n{raw_message})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(fetch_response.as_bytes())
            .expect("FETCH response should be written");

        line.clear();
        reader.read_line(&mut line).expect("LOGOUT should be read");
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

    assert_eq!(batch.emails().len(), 1);
    assert_eq!(
        batch.emails()[0].raw_message,
        "Subject: IMAP письмо\r\nFrom: alice@example.test\r\n\r\nПривет\r\nмир\r\n"
    );

    server.join().expect("IMAP server thread should finish");
}

/// Сценарий mail.ru: SEARCH HEADER отвергнут (NO [CANNOT]),
/// BODY.PEEK[HEADER.FIELDS (...)] отвергнут (BAD [PARSE]).
/// Должен сработать третий уровень fallback: BODY.PEEK[HEADER] (без
/// .FIELDS), затем успешный разбор заголовков в клиенте.
#[test]
fn configured_imap_mailbox_falls_back_to_full_header_when_header_fields_returns_bad_parse() {
    let raw_message =
        "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\nhello\r\n".to_owned();
    let literal_size = raw_message.len();
    let full_headers = "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\n";
    let full_headers_size = full_headers.len();
    let expected_raw_message = raw_message.clone();

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
        socket
            .write_all(b"a001 OK LOGIN completed\r\n")
            .expect("LOGIN response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SELECT should be read");
        socket
            .write_all(b"* 1 EXISTS\r\na002 OK [READ-WRITE] SELECT completed\r\n")
            .expect("SELECT response should be written");

        // 1) SEARCH HEADER → NO [CANNOT]
        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH HEADER should be read");
        socket
            .write_all(b"a003 NO [CANNOT] Unsupported search criterion: X-LIVELETTERS-PROTOCOL\r\n")
            .expect("SEARCH HEADER response should be written");

        // 2) SEARCH ALL → OK, один UID (11)
        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH all should be read");
        assert_eq!(line, "a003 UID SEARCH UID 1:*\r\n");
        socket
            .write_all(b"* SEARCH 11\r\na003 OK SEARCH completed\r\n")
            .expect("SEARCH all response should be written");

        // 3) FETCH HEADER.FIELDS для UID 11 → BAD [PARSE]
        line.clear();
        reader
            .read_line(&mut line)
            .expect("HEADER.FIELDS FETCH for UID 11 should be read");
        assert_eq!(
            line,
            "a004 UID FETCH 11 BODY.PEEK[HEADER.FIELDS (X-LiveLetters-Protocol)]\r\n"
        );
        socket
            .write_all(b"a004 BAD [PARSE] Syntax error while reading parenthesized list\r\n")
            .expect("HEADER.FIELDS FETCH response should be written");

        // 4) FETCH HEADER (без .FIELDS) для UID 11 → OK
        line.clear();
        reader
            .read_line(&mut line)
            .expect("HEADER FETCH for UID 11 should be read");
        assert_eq!(line, "a004 UID FETCH 11 BODY.PEEK[HEADER]\r\n");
        let header_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{full_headers_size}}})\r\n{full_headers})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(header_response.as_bytes())
            .expect("HEADER FETCH response should be written");

        // 5) FETCH BODY[] для UID 11
        line.clear();
        reader
            .read_line(&mut line)
            .expect("BODY FETCH for UID 11 should be read");
        assert_eq!(line, "a004 UID FETCH 11 BODY.PEEK[]\r\n");
        let fetch_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{literal_size}}})\r\n{raw_message})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(fetch_response.as_bytes())
            .expect("BODY FETCH response should be written");

        line.clear();
        reader.read_line(&mut line).expect("LOGOUT should be read");
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
        .fetch_new(&MailboxCursor::from_last_seen_uid(0))
        .expect("real IMAP fetch should succeed via BODY.PEEK[HEADER] fallback");

    assert_eq!(batch.emails().len(), 1);
    assert_eq!(batch.emails()[0].message_id, "imap-uid-11");
    assert_eq!(batch.emails()[0].raw_message, expected_raw_message);
    assert_eq!(batch.next_cursor().last_seen_uid(), Some(11));

    server.join().expect("IMAP server thread should finish");
}

/// Сценарий «совсем экзотический сервер»: и SEARCH HEADER, и
/// BODY.PEEK[HEADER.FIELDS], и BODY.PEEK[HEADER] — все отвергнуты.
/// Должен сработать четвёртый уровень fallback: BODY.PEEK[] (всё
/// тело), и заголовок X-LiveLetters-Protocol парсится в клиенте из
/// тела.
#[test]
fn configured_imap_mailbox_falls_back_to_full_body_when_all_header_fetches_fail() {
    let raw_message =
        "Subject: LiveLetters\r\nX-LiveLetters-Protocol: v1\r\n\r\nhello\r\n".to_owned();
    let literal_size = raw_message.len();
    let expected_raw_message = raw_message.clone();

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
        socket
            .write_all(b"a001 OK LOGIN completed\r\n")
            .expect("LOGIN response should be written");

        line.clear();
        reader.read_line(&mut line).expect("SELECT should be read");
        socket
            .write_all(b"* 1 EXISTS\r\na002 OK [READ-WRITE] SELECT completed\r\n")
            .expect("SELECT response should be written");

        // 1) SEARCH HEADER → BAD
        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH HEADER should be read");
        socket
            .write_all(b"a003 BAD [CANNOT] Unsupported search criterion\r\n")
            .expect("SEARCH HEADER response should be written");

        // 2) SEARCH ALL → OK
        line.clear();
        reader
            .read_line(&mut line)
            .expect("SEARCH all should be read");
        socket
            .write_all(b"* SEARCH 11\r\na003 OK SEARCH completed\r\n")
            .expect("SEARCH all response should be written");

        // 3) FETCH HEADER.FIELDS → BAD
        line.clear();
        reader
            .read_line(&mut line)
            .expect("HEADER.FIELDS FETCH should be read");
        socket
            .write_all(b"a004 BAD [PARSE] Syntax error while reading parenthesized list\r\n")
            .expect("HEADER.FIELDS FETCH response should be written");

        // 4) FETCH HEADER (без .FIELDS) → BAD
        line.clear();
        reader
            .read_line(&mut line)
            .expect("HEADER FETCH should be read");
        socket
            .write_all(b"a004 BAD [PARSE] BODY.PEEK[HEADER] not supported\r\n")
            .expect("HEADER FETCH response should be written");

        // 5) FETCH BODY[] (всё тело) → OK (первый раз — для
        //    extract_liveletters_protocol_header_from_body в fallback'е)
        line.clear();
        reader
            .read_line(&mut line)
            .expect("BODY FETCH should be read");
        let fetch_response = format!(
            "* 1 FETCH (UID 11 BODY[] {{{literal_size}}})\r\n{raw_message})\r\na004 OK FETCH completed\r\n"
        );
        socket
            .write_all(fetch_response.as_bytes())
            .expect("BODY FETCH response should be written");

        // 6) FETCH BODY[] (второй раз — для envelope в fetch_new)
        line.clear();
        reader
            .read_line(&mut line)
            .expect("BODY FETCH (2nd) should be read");
        socket
            .write_all(fetch_response.as_bytes())
            .expect("BODY FETCH (2nd) response should be written");

        line.clear();
        reader.read_line(&mut line).expect("LOGOUT should be read");
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
        .fetch_new(&MailboxCursor::from_last_seen_uid(0))
        .expect("real IMAP fetch should succeed via BODY.PEEK[] fallback");

    assert_eq!(batch.emails().len(), 1);
    assert_eq!(batch.emails()[0].message_id, "imap-uid-11");
    assert_eq!(batch.emails()[0].raw_message, expected_raw_message);
    assert_eq!(batch.next_cursor().last_seen_uid(), Some(11));

    server.join().expect("IMAP server thread should finish");
}
