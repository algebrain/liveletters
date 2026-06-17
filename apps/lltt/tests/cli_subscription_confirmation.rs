//! E2E: B подписывается на A → A автоматически отвечает `SubscriptionConfirmed`
//! → B переводит pending-подписку в подтверждённую, видит ник A.

use assert_cmd::Command;
use tempfile::TempDir;

mod common;

fn lltt() -> Command {
    assert_cmd::Command::cargo_bin("lltt").expect("lltt binary")
}

fn save_user_to_db(home: &std::path::Path, name: &str) {
    let store = liveletters_store::Store::open_for_home_dir(home.join("users").join(name)).unwrap();
    let nickname = match name {
        "alice" => "Алиса",
        "bob" => "Боб",
        other => other,
    };
    let email = format!("{name}@example.org");
    store
        .save_identity(name, &email, nickname, None, "ru", true)
        .unwrap();
}

fn build_subscription_confirmed_eml(from: &str, to: &str, accepted: bool) -> String {
    let message = liveletters_protocol::ProtocolMessage::new(
        liveletters_protocol::MessageEnvelope::new(
            "1",
            "subscription_confirmed",
            from,
            "sub-confirmed-1",
        )
        .unwrap(),
        liveletters_protocol::ProtocolIdentity::new("Алиса".to_owned(), from.to_owned()).unwrap(),
        None,
        if accepted {
            "Подтверждение"
        } else {
            "Отказ"
        },
        liveletters_protocol::DomainEventPayload::SubscriptionConfirmed {
            resource_address: from.into(),
            subscriber_delivery_address: to.into(),
            accepted,
            created_at: 1_710_000_500,
        },
    )
    .unwrap();
    let raw_message = liveletters_mime::build_protocol_email(
        from,
        to,
        "Sync fixture",
        Some(message.human_readable_body().unwrap_or("")),
        &message,
    )
    .unwrap()
    .raw_message;
    // Контракт письма: text/plain непустой, JSON без human_readable_body.
    common::assert_liveletters_email_contract(&raw_message);
    raw_message
}

#[test]
fn subscribe_with_auto_confirmation_moves_pending_to_subscribed() {
    let home = TempDir::new().unwrap();
    common::init_user(home.path(), "alice");
    common::init_user(home.path(), "bob");
    // A должна иметь user_settings, чтобы отправить SubscriptionConfirmed
    save_user_to_db(home.path(), "alice");

    // B подписывается на A → pending
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["sub", "alice@example.org"])
        .assert()
        .success();

    let bob_store =
        liveletters_store::Store::open_for_home_dir(home.path().join("users/bob")).unwrap();
    assert_eq!(
        bob_store.list_pending_subscriptions("bob").unwrap().len(),
        1
    );

    // Симулируем A: получив SubscriptionRequested, A автоматически отвечает
    // SubscriptionConfirmed. Сейчас engine обрабатывает это только если он в
    // фазе ingest. Поэтому делаем ingest вручную через raw eml в bob.

    // Шаг 1: забираем pending из bob — симулируем, что A получил
    // SubscriptionRequested через sync и положил SubscriptionConfirmed в свой
    // outbox.
    // Шаг 2: доставляем SubscriptionConfirmed в bob (как если бы A отправил).
    let confirmed_eml =
        build_subscription_confirmed_eml("alice@example.org", "bob-feed@example.org", true);
    let path = home.path().join("import.eml");
    std::fs::write(&path, &confirmed_eml).unwrap();
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["inbox", "import", path.to_str().unwrap()])
        .assert()
        .success();

    // Теперь: pending пуст, local_subscriptions содержит, authors содержит профиль A.
    let bob_store =
        liveletters_store::Store::open_for_home_dir(home.path().join("users/bob")).unwrap();
    assert!(
        bob_store
            .list_pending_subscriptions("bob")
            .unwrap()
            .is_empty(),
        "pending должен быть удалён"
    );
    let local = bob_store.list_local_subscriptions("bob").unwrap();
    assert!(
        local.contains(&"alice@example.org".to_string()),
        "local_subscriptions должен содержать alice@example.org: {local:?}"
    );
    let author = bob_store
        .get_author("alice@example.org")
        .unwrap()
        .expect("authors должен содержать профиль A");
    assert_eq!(author.nickname, "Алиса");
    assert_eq!(author.source, "subscription_confirmed");
}

#[test]
fn subscribe_declined_removes_pending() {
    let home = TempDir::new().unwrap();
    common::init_user(home.path(), "bob");

    // B подписывается
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["sub", "alice@example.org"])
        .assert()
        .success();

    // A отвечает отказом
    let declined_eml =
        build_subscription_confirmed_eml("alice@example.org", "bob-feed@example.org", false);
    let path = home.path().join("import.eml");
    std::fs::write(&path, &declined_eml).unwrap();
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt()
        .env("LIVELETTERS_HOME", home.path())
        .args(["inbox", "import", path.to_str().unwrap()])
        .assert()
        .success();

    // pending пуст, local пуст
    let bob_store =
        liveletters_store::Store::open_for_home_dir(home.path().join("users/bob")).unwrap();
    assert!(
        bob_store
            .list_pending_subscriptions("bob")
            .unwrap()
            .is_empty()
    );
    assert!(
        bob_store
            .list_local_subscriptions("bob")
            .unwrap()
            .is_empty()
    );
}
