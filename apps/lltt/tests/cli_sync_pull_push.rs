//! e2e тесты `lltt sync`, `lltt sync pull` и `lltt sync push` через бинарь.
//!
//! SMTP-«сервер» поднимается в отдельной нити на `127.0.0.1:0` по
//! идиоме `liveletters-mail/tests/network_flow.rs`. IMAP-имитация
//! ограничена упрощённым вариантом, достаточным для проверки
//! идемпотентности курсора.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use assert_cmd::prelude::*;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("lltt binary")
}

fn init_home(tmp: &TempDir) {
    common::init_user(tmp.path(), "alice");
}

fn set(tmp: &TempDir, key: &str, value: &str) {
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["settings", "set", key, value])
        .assert()
        .success();
}

fn set_mail_settings(tmp: &TempDir, host: &str, port: u16) {
    set_split_mail_settings(tmp, host, port, host, port);
}

fn save_author(store: &liveletters_store::Store, email: &str) {
    store.save_author(email, email, "test").unwrap();
}

fn save_subscription_fixture(store: &liveletters_store::Store, resource: &str, subscriber: &str) {
    save_author(store, resource);
    save_author(store, subscriber);
    store
        .save_subscription(&liveletters_store::SubscriptionRecord {
            resource_email: resource.into(),
            subscriber_email: subscriber.into(),
        })
        .expect("save sub");
}

fn set_split_mail_settings(
    tmp: &TempDir,
    smtp_host: &str,
    smtp_port: u16,
    imap_host: &str,
    imap_port: u16,
) {
    set(tmp, "smtp.host", smtp_host);
    set(tmp, "smtp.port", &smtp_port.to_string());
    set(tmp, "smtp.security", "none");
    set(tmp, "smtp.username", "alice@example.test");
    set(tmp, "smtp.password", "secret");
    set(tmp, "smtp.hello_domain", "local.test");
    set(tmp, "imap.host", imap_host);
    set(tmp, "imap.port", &imap_port.to_string());
    set(tmp, "imap.security", "none");
    set(tmp, "imap.username", "alice");
    set(tmp, "imap.password", "secret");
    set(tmp, "imap.mailbox", "INBOX");
}

/// SMTP-сервер, который принимает несколько соединений подряд,
/// отвечает `250 OK` на все команды и отдаёт собранные `RCPT TO`
/// через receiver. Сервер самозакрывается по `QUIT` от клиента.
fn spawn_fake_smtp(rcpts: Arc<Mutex<Vec<String>>>) -> (String, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();

    let _h = thread::spawn(move || {
        for _ in 0..8 {
            let (mut socket, _) = match listener.accept() {
                Ok(p) => p,
                Err(_) => return,
            };
            socket
                .write_all(b"220 localhost ESMTP capture\r\n")
                .expect("greeting");
            let mut reader = socket.try_clone().expect("clone");
            let mut buf = [0_u8; 8192];
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
                        rcpts.lock().expect("lock").push(addr.to_owned());
                    }
                    if line.starts_with("AUTH PLAIN") {
                        socket
                            .write_all(b"235 2.7.0 Authentication successful\r\n")
                            .ok();
                        continue;
                    }
                    if line.starts_with("DATA") {
                        socket.write_all(b"354 End data\r\n").ok();
                        in_data = true;
                        continue;
                    }
                    if line.starts_with("QUIT") {
                        socket.write_all(b"221 Bye\r\n").ok();
                        return;
                    }
                    socket.write_all(b"250 OK\r\n").ok();
                }
            }
        }
    });
    let _ = _h;
    ("127.0.0.1".to_owned(), port)
}

/// Упрощённый IMAP-сервер: на первое подключение возвращает 1 письмо
/// (из `payload`), на последующие — пустой результат. Следит за
/// счётчиком уже отданных UIDs. Тег в ответе всегда берётся из тега
/// команды клиента (`<tag> OK ...`). Принимает до 5 соединений
/// подряд.
fn spawn_fake_imap(payload: String) -> (String, u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let served = Arc::new(AtomicUsize::new(0));

    let served_clone = Arc::clone(&served);
    let _h = thread::spawn(move || {
        for _ in 0..5 {
            let (mut socket, _) = match listener.accept() {
                Ok(p) => p,
                Err(_) => return,
            };
            socket
                .write_all(b"* OK IMAP4rev1 ready\r\n")
                .expect("greeting");
            let mut reader = socket.try_clone().expect("clone");
            let mut buf = [0_u8; 16384];
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
                    let tag = line.split_whitespace().next().unwrap_or("");
                    if line.contains("LOGIN") {
                        let resp = format!("{tag} OK LOGIN completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                    } else if line.contains("SELECT") {
                        let resp = format!("* 1 EXISTS\r\n{tag} OK SELECT completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                    } else if line.contains("UID SEARCH") {
                        let n_served = served_clone.load(Ordering::SeqCst);
                        let body = if n_served == 0 {
                            "* SEARCH 1\r\n"
                        } else {
                            "* SEARCH\r\n"
                        };
                        let resp = format!("{body}{tag} OK SEARCH completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                    } else if line.contains("UID FETCH") {
                        let literal_size = payload.len();
                        let fetch_response = format!(
                            "* 1 FETCH (UID 1 BODY[] {{{literal_size}}})\r\n{payload}\r\n)\r\n{tag} OK FETCH completed\r\n"
                        );
                        socket.write_all(fetch_response.as_bytes()).ok();
                        served_clone.fetch_add(1, Ordering::SeqCst);
                    } else if line.contains("LOGOUT") {
                        let resp = format!("* BYE\r\n{tag} OK LOGOUT completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                        break; // выходим из inner loop, продолжаем accept
                    }
                }
            }
        }
    });
    let _ = _h;
    ("127.0.0.1".to_owned(), port, served)
}

/// IMAP-сервер, имитирующий поведение Yandex: всегда возвращает
/// `* SEARCH 1` в ответ на любой UID SEARCH, игнорируя `start_uid`.
/// Принимает до 3 соединений подряд.
fn spawn_fake_imap_persistent_uid(payload: String) -> (String, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();

    let _h = thread::spawn(move || {
        for _ in 0..3 {
            let (mut socket, _) = match listener.accept() {
                Ok(p) => p,
                Err(_) => return,
            };
            socket
                .write_all(b"* OK IMAP4rev1 ready\r\n")
                .expect("greeting");
            let mut reader = socket.try_clone().expect("clone");
            let mut buf = [0_u8; 16384];
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
                    let tag = line.split_whitespace().next().unwrap_or("");
                    if line.contains("LOGIN") {
                        let resp = format!("{tag} OK LOGIN completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                    } else if line.contains("SELECT") {
                        let resp = format!("* 1 EXISTS\r\n{tag} OK SELECT completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                    } else if line.contains("UID SEARCH") {
                        let resp = format!("* SEARCH 1\r\n{tag} OK SEARCH completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                    } else if line.contains("UID FETCH") {
                        let literal_size = payload.len();
                        let fetch_response = format!(
                            "* 1 FETCH (UID 1 BODY[] {{{literal_size}}})\r\n{payload}\r\n)\r\n{tag} OK FETCH completed\r\n"
                        );
                        socket.write_all(fetch_response.as_bytes()).ok();
                    } else if line.contains("LOGOUT") {
                        let resp = format!("* BYE\r\n{tag} OK LOGOUT completed\r\n");
                        socket.write_all(resp.as_bytes()).ok();
                        break;
                    }
                }
            }
        }
    });
    let _ = _h;
    ("127.0.0.1".to_owned(), port)
}

#[test]
fn sync_pull_without_mail_settings_returns_helpful_error() {
    let tmp = TempDir::new().expect("tempdir");
    init_home(&tmp);

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "pull"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("настройки почты"),
        "stderr должен сообщать об отсутствии mail_settings; got: {stderr}"
    );
}

#[test]
fn sync_without_subcommand_runs_pull_then_push() {
    let tmp = TempDir::new().expect("tempdir");
    init_home(&tmp);

    use liveletters_mail::build_protocol_email;
    use liveletters_protocol::{
        DomainEventPayload, MessageEnvelope, ProtocolMessage, encode_message,
    };

    let incoming = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-pull-all").unwrap(),
        "Привет из полного sync",
        DomainEventPayload::PostCreated {
            post_id: "post-pull-all".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Привет из полного sync".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let incoming_raw = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Тест",
        Some(incoming.human_readable_body().unwrap_or("")),
        &incoming,
    )
    .expect("build")
    .raw_message;
    // Контракт письма: text/plain непустой, JSON без human_readable_body.
    common::assert_liveletters_email_contract(&incoming_raw);

    let (imap_host, imap_port, served) = spawn_fake_imap(incoming_raw);
    let rcpts = Arc::new(Mutex::new(Vec::<String>::new()));
    let (smtp_host, smtp_port) = spawn_fake_smtp(Arc::clone(&rcpts));
    set_split_mail_settings(&tmp, &smtp_host, smtp_port, &imap_host, imap_port);

    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).expect("store");
    save_subscription_fixture(&store, "blog-1", "bob@example.test");

    let outgoing = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-push-all").unwrap(),
        "Тело",
        DomainEventPayload::PostCreated {
            post_id: "post-push-all".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_100,
            body: "Тело".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    store
        .save_outbox_record(&liveletters_store::OutboxRecord {
            event_id: "event-push-all".into(),
            event_type: "post_created".into(),
            author_email: "blog-1".into(),
            resource_email: Some("blog-1".into()),
            delivery: liveletters_store::OutboxDelivery::ResourceSubscribers,
            message_body: encode_message(&outgoing).expect("encode"),
            message_id: None,
            subject: None,
            human_readable_body: None,
        })
        .expect("save outbox");

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .arg("sync")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("получено писем:       1"),
        "stdout = {stdout}"
    );
    assert!(
        stdout.contains("отправлено писем:     1"),
        "stdout = {stdout}"
    );
    assert!(served.load(Ordering::SeqCst) >= 1);
    assert!(
        rcpts
            .lock()
            .expect("lock")
            .contains(&"bob@example.test".to_owned())
    );
    assert!(store.list_outbox_records().expect("outbox").is_empty());
}

#[test]
fn sync_push_sends_one_email_per_subscriber_and_clears_outbox() {
    let tmp = TempDir::new().expect("tempdir");
    init_home(&tmp);

    let rcpts = Arc::new(Mutex::new(Vec::<String>::new()));
    let (host, port) = spawn_fake_smtp(Arc::clone(&rcpts));
    set_mail_settings(&tmp, &host, port);

    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).expect("store");
    save_subscription_fixture(&store, "blog-1", "bob@example.test");

    use liveletters_protocol::{
        DomainEventPayload, MessageEnvelope, ProtocolMessage, encode_message,
    };
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-push-1").unwrap(),
        "Тело",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Тело".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let body = encode_message(&message).expect("encode");
    store
        .save_outbox_record(&liveletters_store::OutboxRecord {
            event_id: "event-push-1".into(),
            event_type: "post_created".into(),
            author_email: "blog-1".into(),
            resource_email: Some("blog-1".into()),
            delivery: liveletters_store::OutboxDelivery::ResourceSubscribers,
            message_body: body,
            message_id: None,
            subject: None,
            human_readable_body: None,
        })
        .expect("save outbox");

    let assert = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "push"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stdout.contains("отправлено писем:     1"),
        "push stdout = {stdout}\npush stderr = {stderr}"
    );

    let remaining = store.list_outbox_records().expect("list outbox");
    assert!(
        remaining.is_empty(),
        "outbox должен быть очищен, осталось {remaining:?}"
    );

    let collected = rcpts.lock().expect("lock").clone();
    assert!(
        collected.contains(&"bob@example.test".to_owned()),
        "SMTP должен был получить RCPT TO:<bob@example.test>, got {collected:?}"
    );

    let _ = Duration::from_millis;
}

#[test]
fn sync_pull_advances_cursor_idempotently() {
    let tmp = TempDir::new().expect("tempdir");
    init_home(&tmp);

    use liveletters_mail::build_protocol_email;
    use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-pull-1").unwrap(),
        "Привет из IMAP",
        DomainEventPayload::PostCreated {
            post_id: "post-1".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Привет из IMAP".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Тест",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("build");
    let raw = outgoing.raw_message;
    // Контракт письма: text/plain непустой, JSON без human_readable_body.
    common::assert_liveletters_email_contract(&raw);

    let (host, port, served) = spawn_fake_imap(raw);
    set_mail_settings(&tmp, &host, port);
    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).expect("store");
    save_author(&store, "blog-1");

    let assert1 = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "pull"])
        .assert()
        .success();
    let stdout1 = String::from_utf8_lossy(&assert1.get_output().stdout);
    assert!(
        stdout1.contains("получено писем:       1"),
        "first stdout = {stdout1}"
    );

    // Даём серверу время на полную обработку первого соединения.
    thread::sleep(Duration::from_millis(50));

    let assert2 = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "pull"])
        .assert()
        .success();
    let stdout2 = String::from_utf8_lossy(&assert2.get_output().stdout);
    assert!(
        stdout2.contains("получено писем:       0"),
        "second stdout = {stdout2}"
    );
    assert!(
        served.load(Ordering::SeqCst) >= 1,
        "IMAP-сервер должен был отдать как минимум 1 FETCH"
    );
}

#[test]
fn sync_push_with_direct_delivery_ignores_subscriptions_table() {
    let tmp = TempDir::new().expect("tempdir");
    init_home(&tmp);

    let rcpts = Arc::new(Mutex::new(Vec::<String>::new()));
    let (host, port) = spawn_fake_smtp(Arc::clone(&rcpts));
    set_mail_settings(&tmp, &host, port);

    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).expect("store");

    save_subscription_fixture(&store, "algebrain@example.org", "alice@example.test");

    use liveletters_protocol::{
        DomainEventPayload, MessageEnvelope, ProtocolMessage, encode_message,
    };
    let outgoing = ProtocolMessage::new(
        MessageEnvelope::new(
            "1",
            "subscription_requested",
            "algebrain@example.org",
            "event-sub-direct",
        )
        .unwrap(),
        "Запрос подписки",
        DomainEventPayload::SubscriptionRequested {
            resource_address: "algebrain@example.org".into(),
            subscriber_delivery_address: "alice@example.test".into(),
            subscriber_nickname: "Алиса".into(),
            created_at: 1_710_000_000,
        },
    )
    .unwrap();
    store
        .save_outbox_record(&liveletters_store::OutboxRecord {
            event_id: "event-sub-direct".into(),
            event_type: "subscription_requested".into(),
            author_email: "alice@example.test".into(),
            resource_email: Some("algebrain@example.org".into()),
            delivery: liveletters_store::OutboxDelivery::Direct(vec![
                "algebrain@example.org".into(),
            ]),
            message_body: encode_message(&outgoing).expect("encode"),
            message_id: None,
            subject: None,
            human_readable_body: None,
        })
        .expect("save outbox");

    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "push"])
        .assert()
        .success();

    let collected = rcpts.lock().expect("lock").clone();
    assert!(
        collected.contains(&"algebrain@example.org".to_owned()),
        "RCPT TO должен содержать algebrain@example.org, got {collected:?}"
    );
    assert!(
        !collected.contains(&"alice@example.test".to_owned()),
        "RCPT TO не должен содержать alice@example.test (это подписчик, не адресат), got {collected:?}"
    );

    assert!(store.list_outbox_records().expect("outbox").is_empty());
}

#[test]
fn sync_pull_re_fetches_same_uid_when_imap_ignores_start_uid() {
    let tmp = TempDir::new().expect("tempdir");
    init_home(&tmp);

    use liveletters_mail::build_protocol_email;
    use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", "event-pull-2").unwrap(),
        "Привет из IMAP",
        DomainEventPayload::PostCreated {
            post_id: "post-2".into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: "Привет из IMAP".into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();
    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Тест",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("build");
    let raw = outgoing.raw_message;
    // Контракт письма: text/plain непустой, JSON без human_readable_body.
    common::assert_liveletters_email_contract(&raw);

    let (host, port) = spawn_fake_imap_persistent_uid(raw);
    set_mail_settings(&tmp, &host, port);
    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).expect("store");
    save_author(&store, "blog-1");

    let assert1 = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "pull"])
        .assert()
        .success();
    let stdout1 = String::from_utf8_lossy(&assert1.get_output().stdout);
    assert!(
        stdout1.contains("получено писем:       1"),
        "first sync should receive the email; stdout = {stdout1}"
    );

    thread::sleep(Duration::from_millis(50));

    let assert2 = lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["sync", "pull"])
        .assert()
        .success();
    let stdout2 = String::from_utf8_lossy(&assert2.get_output().stdout);
    assert!(
        stdout2.contains("получено писем:       0"),
        "RED: second sync should receive 0 emails after cursor advance, but IMAP ignores start_uid; stdout = {stdout2}"
    );
}
