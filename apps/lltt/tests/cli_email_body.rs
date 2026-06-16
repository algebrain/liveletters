//! E2E: созданный пост в `lltt post new` порождает email с непустым
//! локализованным `text/plain` телом. Это критическая runtime-проверка
//! того, что человекочитаемое тело письма живо (оно намеренно не
//! сериализуется в JSON — хранится в отдельной колонке
//! `OutboxRecord.human_readable_body` и кладётся в `text/plain` при
//! сборке письма).

use std::fs;

use liveletters_mime::build_protocol_email;
use liveletters_protocol::decode_message;
use tempfile::TempDir;

mod common;

fn lltt() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("lltt").expect("бинарь lltt")
}

#[test]
fn post_new_email_has_non_empty_localized_text_plain_body() {
    let tmp = TempDir::new().unwrap();
    common::init_user(tmp.path(), "alice");

    let body_path = tmp.path().join("body.txt");
    fs::write(&body_path, "Привет, мир!").unwrap();
    lltt()
        .env("LIVELETTERS_HOME", tmp.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();

    let store =
        liveletters_store::Store::open_for_home_dir(tmp.path().join("users/alice")).unwrap();
    let records = store.list_outbox_records().unwrap();
    assert_eq!(records.len(), 1, "должна быть ровно 1 outbox-запись");
    let record = &records[0];

    // Человекочитаемое тело теперь хранится в отдельной колонке outbox,
    // а в JSON его нет (см. message.rs: skip_serializing).
    let human_body = record
        .human_readable_body
        .as_deref()
        .expect("OutboxRecord.human_readable_body должен быть заполнен");

    // Собираем email точно так же, как это делает send_outbox_record.
    let message = decode_message(&record.message_body).unwrap();
    let subject = record.subject.as_deref().unwrap_or(&record.event_type);
    let outgoing = build_protocol_email(
        "alice-publish@example.org",
        "subscriber@example.org",
        subject,
        Some(human_body),
        &message,
    )
    .expect("сборка email не должна падать");

    // text/plain под-часть — не пустая, локализованная.
    assert!(
        outgoing.raw_message.contains("Новая запись в журнале"),
        "text/plain должен содержать локализованный заголовок: {}",
        outgoing.raw_message
    );
    assert!(
        outgoing.raw_message.contains("Привет, мир!"),
        "text/plain должен содержать тело поста: {}",
        outgoing.raw_message
    );

    // В JSON-поле human_readable_body отсутствует (проверяем, что
    // дублирования в wire-формате нет).
    let json: serde_json::Value = serde_json::from_str(&record.message_body).unwrap();
    assert!(
        json.get("human_readable_body").is_none(),
        "human_readable_body не должно попадать в JSON: {json}"
    );
}
