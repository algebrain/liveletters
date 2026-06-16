//! Общие фикстуры для интеграционных тестов бинаря `lltt`.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;
use liveletters_mime::{build_protocol_email, extract_liveletters_parts, parse_email};
use liveletters_protocol::{DomainEventPayload, MessageEnvelope, ProtocolMessage};

pub fn lltt() -> Command {
    Command::cargo_bin("lltt").expect("бинарь lltt")
}

pub fn init_user(home: &std::path::Path, name: &str) {
    lltt()
        .env("LIVELETTERS_HOME", home)
        .args(["init", "--force"])
        .assert()
        .success();
    write_identity(home, name);
    lltt()
        .env("LIVELETTERS_HOME", home)
        .args(["cu", name])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "language", "ru"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "nickname", name])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "email_address", &format!("{name}@example.org")])
        .assert()
        .success();
}

pub fn write_identity(home: &std::path::Path, name: &str) {
    std::fs::create_dir_all(home.join("identities")).expect("create identities");
    std::fs::write(
        home.join("identities").join(format!("{name}.toml")),
        format!(
            r#"
display_name = "{name}"

[mail]
publish = "{name}@example.org"
receive = ["{name}@example.org"]

[meta]
resources_owned = ["{name}@example.org"]
subscriptions = []
"#
        ),
    )
    .expect("write identity");
}

pub fn write_post_eml(dir: &std::path::Path, post_id: &str, body: &str) -> PathBuf {
    let message = ProtocolMessage::new(
        MessageEnvelope::new("1", "post_created", "blog-1", &format!("event-{post_id}")).unwrap(),
        body,
        DomainEventPayload::PostCreated {
            post_id: post_id.into(),
            resource_id: "blog-1".into(),
            actor_id: "alice".into(),
            created_at: 1_710_000_000,
            body: body.into(),
            body_format: "plain".into(),
            visibility: "public".into(),
        },
    )
    .unwrap();

    let outgoing = build_protocol_email(
        "alice@example.test",
        "bob@example.test",
        "Запись",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .expect("raw email builds");

    let with_id = outgoing.raw_message.replacen(
        "Subject: =?utf-8?B?0JfQsNC/0LjRgdGM?=\n",
        &format!("Subject: =?utf-8?B?0JfQsNC/0LjRgdGM?=\nMessage-ID: <{post_id}@example.test>\n"),
        1,
    );
    let path = dir.join(format!("{post_id}.eml"));
    std::fs::write(&path, with_id).expect("write eml");
    path
}

/// Проверяет, что собранное LiveLetters-письмо удовлетворяет контракту
/// `human_readable_body` (см. `.plans/260616-144732-split-human-readable-body.md`):
///
/// 1. `text/plain` под-часть не пустая — там лежит локализованный
///    текст письма, который видит пользователь в своём почтовом клиенте.
/// 2. `application/json` (он же `liveletters.json`) **не** содержит поля
///    `human_readable_body` — тело хранится в отдельной колонке outbox,
///    в wire-формате дублирования быть не должно.
///
/// Вызывается из всех e2e-тестов, где письмо собирается через
/// `build_protocol_email` и доставляется в `inbox import`. Без этой
/// проверки регрессия (как в `260616-144732`) прошла бы незамеченной —
/// поле `human_readable_body` могло бы снова «протечь» в JSON или,
/// наоборот, исчезнуть из `text/plain`.
pub fn assert_liveletters_email_contract(raw_message: &str) {
    let parsed = parse_email(raw_message).expect("parse email");
    let parts = extract_liveletters_parts(&parsed).expect("extract liveletters parts");

    // 1. text/plain не пустой.
    let plain = parts.human_readable_body();
    assert!(
        !plain.trim().is_empty(),
        "text/plain под-часть должна быть непустой. raw_email:\n{raw_message}"
    );

    // 2. JSON не содержит human_readable_body.
    let json: serde_json::Value =
        serde_json::from_str(parts.technical_body()).expect("parse JSON из email");
    assert!(
        json.get("human_readable_body").is_none(),
        "JSON не должен содержать поле human_readable_body. JSON: {json}"
    );
}

/// Пишет в `dir/<name>.eml` «невалидное» письмо: валидный envelope,
/// но тело — `text/plain` без второй `application/json` части.
/// `SyncEngine` парсит multipart, не находит JSON, кладёт в
/// `raw_messages` строку со `status="malformed"` и возвращает
/// `SyncMessageOutcome::Malformed` → `lltt doctor` показывает
/// `здоровье: деградирован` и `Malformed: 1`.
pub fn write_malformed_post_eml(dir: &std::path::Path, name: &str) -> PathBuf {
    let raw = format!(
        "From: alice <alice@example.test>\n\
         To: bob@example.test\n\
         Subject: {name}\n\
         Message-ID: <{name}@example.test>\n\
         MIME-Version: 1.0\n\
         Content-Type: text/plain; charset=utf-8\n\
         \n\
         This is plain text, not a multipart with application/json.\n"
    );
    let path = dir.join(format!("{name}.eml"));
    std::fs::write(&path, raw).expect("write malformed eml");
    path
}
