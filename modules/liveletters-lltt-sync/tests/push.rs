use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Barrier, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use liveletters_mail::{
    ConfiguredSmtpTransport, MailAuth, MailSecurity, SmtpTransportConfig, build_protocol_email,
};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage, encode_message};
use liveletters_store::{OutboxDelivery, OutboxRecord, Store, SubscriptionRecord};
use tempfile::TempDir;

use liveletters_lltt_sync::send_outbox_record;

fn open_store() -> (TempDir, Store) {
    let tmp = TempDir::new().expect("tempdir");
    let store = Store::open_for_home_dir(tmp.path()).expect("store opens");
    (tmp, store)
}

fn sample_protocol_message(event_id: &str) -> ProtocolMessage {
    ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", event_id).unwrap(),
        "Тестовое письмо",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Тестовое письмо".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap()
}

fn outbox_record_for(message: &ProtocolMessage) -> OutboxRecord {
    OutboxRecord {
        event_id: message.envelope().event_id().to_owned(),
        event_type: message.envelope().event_type().to_owned(),
        resource_id: message.envelope().resource_id().to_owned(),
        delivery: OutboxDelivery::ResourceSubscribers,
        message_body: encode_message(message).expect("protocol serializes"),
        message_id: None,
        subject: None,
    }
}

/// Поднимает «курьерский» SMTP-сервер, который принимает несколько
/// соединений подряд. Для каждого отдаёт `220`, отвечает `250 OK`
/// на каждую команду, на `QUIT` отвечает `221 Bye` и закрывает
/// **текущий сокет** (но не listener). Собранные `RCPT TO` со всех
/// сессий отдаются через `receiver`.
fn spawn_smtp_capture() -> (String, u16, mpsc::Receiver<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let (tx, rx) = mpsc::channel();
    let ready = Arc::new(Barrier::new(2));
    let rcpts = Arc::new(Mutex::new(Vec::<String>::new()));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let ready_clone = Arc::clone(&ready);
    let rcpts_clone = Arc::clone(&rcpts);
    let stop_clone = Arc::clone(&stop);
    let _handle = thread::spawn(move || {
        let _ = ready_clone.wait();
        while !stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
            let (mut socket, _) = match listener.accept() {
                Ok(p) => p,
                Err(_) => return,
            };
            socket
                .write_all(b"220 localhost ESMTP capture\r\n")
                .expect("greeting");

            let mut reader = socket.try_clone().expect("clone");
            let mut buf = [0_u8; 4096];
            let mut in_data = false;
            loop {
                let n = match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };
                let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                for line in chunk.split("\r\n") {
                    if line.is_empty() {
                        continue;
                    }
                    if in_data {
                        if line == "." {
                            socket.write_all(b"250 OK\r\n").ok();
                            in_data = false;
                        }
                        continue;
                    }
                    if let Some(rcpt) = line.strip_prefix("RCPT TO:<")
                        && let Some(addr) = rcpt.strip_suffix('>')
                    {
                        rcpts_clone
                            .lock()
                            .expect("rcpts lock")
                            .push(addr.to_owned());
                    }
                    if line.starts_with("DATA") {
                        socket.write_all(b"354 End data\r\n").ok();
                        in_data = true;
                        continue;
                    }
                    if line.starts_with("QUIT") {
                        socket.write_all(b"221 Bye\r\n").ok();
                        break;
                    }
                    socket.write_all(b"250 OK\r\n").ok();
                }
            }
        }
    });

    let _ = ready.wait();
    thread::sleep(Duration::from_millis(20));
    let rcpts_clone2 = Arc::clone(&rcpts);
    let stop_clone2 = Arc::clone(&stop);
    let tx_clone = tx;
    let _collector = thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            let n = rcpts_clone2.lock().expect("lock").len();
            if n >= 2 {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        stop_clone2.store(true, std::sync::atomic::Ordering::Relaxed);
        let collected = std::mem::take(&mut *rcpts_clone2.lock().expect("lock"));
        let _ = tx_clone.send(collected);
    });
    let _ = _collector;
    ("127.0.0.1".to_owned(), port, rx)
}

/// SMTP-сервер, который дополнительно собирает Subject каждого
/// отправленного письма. Возвращает `rx_subjects` в том же порядке,
/// что и `rx_rcpts`. Используется в тестах на локализованный Subject.
fn spawn_smtp_capture_subjects() -> (String, u16, mpsc::Receiver<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let (tx, rx) = mpsc::channel();
    let ready = Arc::new(Barrier::new(2));
    let subjects = Arc::new(Mutex::new(Vec::<String>::new()));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let ready_clone = Arc::clone(&ready);
    let subjects_clone = Arc::clone(&subjects);
    let stop_clone = Arc::clone(&stop);
    let _handle = thread::spawn(move || {
        let _ = ready_clone.wait();
        while !stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
            let (mut socket, _) = match listener.accept() {
                Ok(p) => p,
                Err(_) => return,
            };
            socket
                .write_all(b"220 localhost ESMTP capture\r\n")
                .expect("greeting");
            let mut reader = socket.try_clone().expect("clone");
            let mut buf = [0_u8; 8192];
            let mut data_buf = String::new();
            let mut in_data = false;
            let mut last_subject: Option<String> = None;
            loop {
                let n = match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };
                let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                for line in chunk.split("\r\n") {
                    if line.is_empty() {
                        continue;
                    }
                    if in_data {
                        if line == "." {
                            if let Some(s) = last_subject.take() {
                                subjects_clone.lock().expect("subjects lock").push(s);
                            }
                            socket.write_all(b"250 OK\r\n").ok();
                            in_data = false;
                            data_buf.clear();
                        } else {
                            data_buf.push_str(line);
                            data_buf.push('\n');
                            if let Some(s) = line.strip_prefix("Subject: ").map(str::to_owned) {
                                last_subject = Some(s);
                            }
                        }
                        continue;
                    }
                    if line.starts_with("DATA") {
                        socket.write_all(b"354 End data\r\n").ok();
                        in_data = true;
                        continue;
                    }
                    if line.starts_with("QUIT") {
                        socket.write_all(b"221 Bye\r\n").ok();
                        break;
                    }
                    socket.write_all(b"250 OK\r\n").ok();
                }
            }
        }
    });

    let _ = ready.wait();
    thread::sleep(Duration::from_millis(20));
    let subjects_clone2 = Arc::clone(&subjects);
    let stop_clone2 = Arc::clone(&stop);
    let tx_clone = tx;
    let _collector = thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            let n = subjects_clone2.lock().expect("lock").len();
            if n >= 1 {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        stop_clone2.store(true, std::sync::atomic::Ordering::Relaxed);
        let collected = std::mem::take(&mut *subjects_clone2.lock().expect("lock"));
        let _ = tx_clone.send(collected);
    });
    let _ = _collector;
    ("127.0.0.1".to_owned(), port, rx)
}

fn make_transport(port: u16) -> ConfiguredSmtpTransport {
    ConfiguredSmtpTransport::new(SmtpTransportConfig::new(
        "127.0.0.1",
        port,
        "local.test",
        MailSecurity::None,
        MailAuth::None,
    ))
}

#[test]
fn resource_subscribers_sends_one_email_per_subscriber() {
    let (_tmp, store) = open_store();
    let (_host, port, rx) = spawn_smtp_capture();

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "blog-1".into(),
            subscriber_delivery_address: "bob@example.test".into(),
        })
        .expect("save sub bob");
    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "blog-1".into(),
            subscriber_delivery_address: "carol@example.test".into(),
        })
        .expect("save sub carol");

    let message = sample_protocol_message("event-1");
    let record = outbox_record_for(&message);

    let transport = make_transport(port);
    let n = send_outbox_record(&store, &transport, "alice@example.test", &record)
        .expect("send to subscribers");
    assert_eq!(n, 2);

    let collected = rx.recv().expect("rcpts collected");
    assert_eq!(
        collected.len(),
        2,
        "RCPT TO должно быть по одному на подписчика"
    );
    assert!(collected.contains(&"bob@example.test".to_owned()));
    assert!(collected.contains(&"carol@example.test".to_owned()));
}

#[test]
fn resource_subscribers_skips_when_no_subscribers() {
    let (_tmp, store) = open_store();
    let (_host, port, _rx) = spawn_smtp_capture();
    let transport = make_transport(port);

    let message = sample_protocol_message("event-2");
    let record = outbox_record_for(&message);
    let n = send_outbox_record(&store, &transport, "alice@example.test", &record)
        .expect("no subs is not an error");
    assert_eq!(n, 0);
}

#[test]
fn resource_subscribers_propagates_smtp_error() {
    let (_tmp, store) = open_store();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "blog-1".into(),
            subscriber_delivery_address: "bob@example.test".into(),
        })
        .expect("save sub");

    let message = sample_protocol_message("event-3");
    let record = outbox_record_for(&message);
    let transport = make_transport(port);

    let result = send_outbox_record(&store, &transport, "alice@example.test", &record);
    assert!(result.is_err(), "SMTP-сбой должен пробрасываться");
}

#[test]
fn direct_delivery_sends_only_to_declared_recipients() {
    let (_tmp, store) = open_store();
    let (_host, port, rx) = spawn_smtp_capture();

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "blog-1".into(),
            subscriber_delivery_address: "bob@example.test".into(),
        })
        .expect("save sub");

    let message = sample_protocol_message("event-direct");
    let record = OutboxRecord {
        delivery: OutboxDelivery::Direct(vec!["algebrain@example.org".into()]),
        ..outbox_record_for(&message)
    };

    let transport = make_transport(port);
    let n =
        send_outbox_record(&store, &transport, "alice@example.test", &record).expect("direct send");
    assert_eq!(n, 1, "должно быть отправлено ровно одно письмо");

    let collected = rx.recv().expect("rcpts collected");
    assert_eq!(collected.len(), 1);
    assert!(collected.contains(&"algebrain@example.org".to_owned()));
    assert!(!collected.contains(&"bob@example.test".to_owned()));
}

#[test]
fn direct_delivery_sends_to_multiple_addresses_in_order() {
    let (_tmp, store) = open_store();
    let (_host, port, rx) = spawn_smtp_capture();

    let message = sample_protocol_message("event-multi");
    let record = OutboxRecord {
        delivery: OutboxDelivery::Direct(vec!["x@example.org".into(), "y@example.org".into()]),
        ..outbox_record_for(&message)
    };

    let transport = make_transport(port);
    let n = send_outbox_record(&store, &transport, "alice@example.test", &record)
        .expect("multi direct send");
    assert_eq!(n, 2);

    let collected = rx.recv().expect("rcpts collected");
    assert_eq!(collected.len(), 2);
    assert!(collected.contains(&"x@example.org".to_owned()));
    assert!(collected.contains(&"y@example.org".to_owned()));
}

#[test]
fn build_protocol_email_round_trip_for_push() {
    let message = sample_protocol_message("event-rt");
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        message.envelope().event_type(),
        &message,
    )
    .expect("build");
    assert!(outgoing.raw_message.contains("Subject: post_created"));
}

#[test]
fn push_uses_localized_subject_when_present() {
    let (_tmp, store) = open_store();
    let (_host, port, rx_subjects) = spawn_smtp_capture_subjects();
    let transport = make_transport(port);

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "blog-1".into(),
            subscriber_delivery_address: "bob@example.test".into(),
        })
        .expect("save sub");

    let message = sample_protocol_message("event-loc");
    let mut record = outbox_record_for(&message);
    // event_type — технический, subject — локализованный (новая конвенция)
    record.event_type = "post_created".to_owned();
    record.subject = Some("Новая запись в журнале blog-1".to_owned());

    let _ = send_outbox_record(&store, &transport, "alice@example.test", &record).expect("send ok");
    let subjects = rx_subjects.recv().expect("subjects collected");
    assert_eq!(subjects.len(), 1, "ожидался 1 subject");
    // Subject в SMTP-капчуре приходит в виде RFC 2047 `=?utf-8?B?...?=`;
    // проверяем наличие base64 от ожидаемой строки.
    let expected_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        "Новая запись в журнале blog-1".as_bytes(),
    );
    assert!(
        subjects[0].contains(&expected_b64),
        "в письме должен быть локализованный subject (RFC 2047), получили: {:?}",
        subjects[0]
    );
    assert!(
        !subjects[0].contains("post_created"),
        "в письме не должно быть технического идентификатора: {:?}",
        subjects[0]
    );
}

#[test]
fn push_falls_back_to_event_type_when_subject_missing() {
    let (_tmp, store) = open_store();
    let (_host, port, rx_subjects) = spawn_smtp_capture_subjects();
    let transport = make_transport(port);

    store
        .save_subscription(&SubscriptionRecord {
            resource_address: "blog-1".into(),
            subscriber_delivery_address: "bob@example.test".into(),
        })
        .expect("save sub");

    let message = sample_protocol_message("event-fallback");
    let record = outbox_record_for(&message); // subject: None

    let _ = send_outbox_record(&store, &transport, "alice@example.test", &record).expect("send ok");
    let subjects = rx_subjects.recv().expect("subjects collected");
    assert_eq!(subjects.len(), 1);
    // fallback на event_type для обратной совместимости
    assert!(
        subjects[0].contains("post_created"),
        "fallback должен дать event_type, получили: {:?}",
        subjects[0]
    );
}
