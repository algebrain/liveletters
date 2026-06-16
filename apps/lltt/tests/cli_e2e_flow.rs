//! Полные e2e-сценарии с подтверждением подписки и пересылкой.

use assert_cmd::Command;
use liveletters_protocol::DomainEventPayload;
use tempfile::TempDir;

fn lltt_cmd() -> Command {
    assert_cmd::Command::cargo_bin("lltt").expect("lltt binary")
}

fn write_identity(home: &std::path::Path, name: &str) {
    std::fs::create_dir_all(home.join("identities")).expect("identities dir");
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
"#
        ),
    )
    .expect("write identity");
}

fn setup_user(home: &std::path::Path, name: &str) {
    write_identity(home, name);
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["cu", name])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "language", "ru"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "nickname", name])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["set", "email_address", &format!("{name}@example.org")])
        .assert()
        .success();
}

fn open_user_store(home: &std::path::Path, user: &str) -> liveletters_store::Store {
    liveletters_store::Store::open_for_home_dir(home.join("users").join(user))
        .expect("open user store")
}

fn outbox_records(home: &std::path::Path, user: &str) -> Vec<liveletters_store::OutboxRecord> {
    open_user_store(home, user)
        .list_outbox_records()
        .expect("list outbox")
}

fn find_outbox<'a>(
    records: &'a [liveletters_store::OutboxRecord],
    envelope_event_type: &str,
) -> Option<&'a liveletters_store::OutboxRecord> {
    records.iter().find(|r| {
        liveletters_protocol::decode_message(&r.message_body)
            .ok()
            .map(|m| m.envelope().event_type() == envelope_event_type)
            .unwrap_or(false)
    })
}

fn build_direct_eml(from: &str, to: &str, record: &liveletters_store::OutboxRecord) -> String {
    let message = liveletters_protocol::decode_message(&record.message_body)
        .expect("decode outbox message_body");
    liveletters_mime::build_protocol_email(from, to, &record.event_type, &message)
        .expect("build email")
        .raw_message
}

fn import_eml(home: &std::path::Path, raw_message: &str) {
    let path = home.join("import.eml");
    std::fs::write(&path, raw_message).expect("write import eml");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["inbox", "import", path.to_str().unwrap()])
        .assert()
        .success();
}

fn import_all_direct_emails(
    home: &std::path::Path,
    user: &str,
    from: &str,
    envelope_event_type: &str,
) -> Vec<String> {
    let records = outbox_records(home, user);
    let Some(record) = find_outbox(&records, envelope_event_type) else {
        panic!(
            "ожидалась outbox-запись с envelope.event_type={envelope_event_type:?} у {user}, \
             есть: {:?}",
            records
                .iter()
                .map(|r| {
                    liveletters_protocol::decode_message(&r.message_body)
                        .ok()
                        .map(|m| m.envelope().event_type().to_owned())
                        .unwrap_or_else(|| "<не декодируется>".to_owned())
                })
                .collect::<Vec<_>>()
        );
    };
    // Собираем список адресатов: либо из `Direct`, либо — если
    // `ResourceSubscribers` — резолвим в подписчиков ресурса из БД
    // отправителя (потому что отправитель — владелец ресурса и знает
    // всех своих подписчиков).
    let recipients: Vec<String> = match &record.delivery {
        liveletters_store::OutboxDelivery::Direct(addrs) => addrs.clone(),
        liveletters_store::OutboxDelivery::ResourceSubscribers => {
            // resource_id берём из payload.
            let resource_id = match liveletters_protocol::decode_message(&record.message_body)
                .expect("decode message")
                .payload()
            {
                liveletters_protocol::DomainEventPayload::PostCreated { resource_id, .. } => {
                    resource_id.clone()
                }
                liveletters_protocol::DomainEventPayload::CommentCreated {
                    resource_id, ..
                } => resource_id.clone(),
                other => panic!(
                    "ResourceSubscribers ожидался только для PostCreated/CommentCreated, \
                     а у нас {other:?}"
                ),
            };
            let store = open_user_store(home, user);
            let subs = store
                .list_subscriptions_for_resource(&resource_id)
                .expect("list subscriptions for resource");
            subs.into_iter()
                .map(|s| s.subscriber_delivery_address)
                .collect()
        }
    };
    assert!(
        !recipients.is_empty(),
        "{envelope_event_type} у {user}: список адресатов пуст"
    );
    let mut out = Vec::new();
    for to in recipients {
        let eml = build_direct_eml(from, &to, record);
        let target = match to.as_str() {
            "alice@example.org" => "alice",
            "bob@example.org" => "bob",
            "eve@example.org" => "eve",
            other => panic!("import_all_direct_emails: неизвестный адресат {other}"),
        };
        lltt_cmd()
            .env("LIVELETTERS_HOME", home)
            .args(["cu", target])
            .assert()
            .success();
        import_eml(home, &eml);
        out.push(eml);
    }
    out
}

/// B подписывается на A, A автоматически отвечает `SubscriptionConfirmed`,
/// B принимает. После этого B видит A в `local_subscriptions` и
/// `display_names` (ник A).
///
/// Это утилита для тестов, которые полагались на старое поведение
/// «B сразу подписан после `lltt sub`».
fn subscribe_and_confirm(home: &std::path::Path, subscriber: &str, target: &str) {
    // B отправляет SubscriptionRequested
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["cu", subscriber])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home)
        .args(["sub", &format!("{target}@example.org")])
        .assert()
        .success();

    // Доставляем SubscriptionRequested в A — A автоматически ответит
    // SubscriptionConfirmed и положит в свой outbox.
    // `import_all_direct_emails` итерирует по всем записям этого типа в
    // outbox подписчика и доставляет каждой в своего адресата; после
    // первого вызова alice уже отправит `SubscriptionConfirmed` конкретно
    // этому подписчику, и второй вызов ниже заберёт именно её.
    import_all_direct_emails(
        home,
        subscriber,
        &format!("{subscriber}@example.org"),
        "subscription_requested",
    );

    // Забираем ВСЕ SubscriptionConfirmed из outbox A и доставляем их
    // адресатам. Это идемпотентно для уже подтверждённых подписчиков и
    // корректно обрабатывает несколько подписчиков одного ресурса.
    let confirmed_records: Vec<liveletters_store::OutboxRecord> = outbox_records(home, target)
        .into_iter()
        .filter(|r| {
            liveletters_protocol::decode_message(&r.message_body)
                .ok()
                .map(|m| m.envelope().event_type() == "subscription_confirmed")
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !confirmed_records.is_empty(),
        "alice должна положить subscription_confirmed в outbox для {subscriber}"
    );
    for r in &confirmed_records {
        let to = match &r.delivery {
            liveletters_store::OutboxDelivery::Direct(addrs) => addrs
                .first()
                .cloned()
                .expect(" Direct с пустым списком — недопустимо"),
            liveletters_store::OutboxDelivery::ResourceSubscribers => {
                panic!("subscription_confirmed не должен быть ResourceSubscribers")
            }
        };
        // Доставляем только если адресат — наш текущий подписчик.
        if to != format!("{subscriber}@example.org") {
            continue;
        }
        let eml = build_direct_eml(&format!("{target}@example.org"), &to, r);
        lltt_cmd()
            .env("LIVELETTERS_HOME", home)
            .args(["cu", subscriber])
            .assert()
            .success();
        import_eml(home, &eml);
    }

    // Проверяем, что B теперь подписан (local_subscriptions содержит target).
    let store = open_user_store(home, subscriber);
    let local = store.list_local_subscriptions(subscriber).unwrap();
    let target_email = format!("{target}@example.org");
    assert!(
        local.contains(&target_email),
        "B должен быть подписан на {target_email} после confirm; local={local:?}"
    );
}

#[test]
fn two_users_subscribe_posts_appear_in_feed() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["init", "--force"])
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");

    // bob подписывается на alice (с подтверждением)
    subscribe_and_confirm(home.path(), "bob", "alice");

    // alice создаёт пост
    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Первый пост").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let _post_id = open_user_store(home.path(), "alice").list_posts().unwrap()[0]
        .post_id
        .clone();

    // доставляем пост в bob
    import_all_direct_emails(home.path(), "alice", "alice@example.org", "post_created");

    // проверка: bob видит пост в feed
    let assert = lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("feed")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Первый пост"),
        "feed должен содержать 'Первый пост':\n{stdout}"
    );
}

#[test]
fn alice_comments_own_post_subscriber_sees_it() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["init", "--force"])
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");

    // bob подписывается на alice (с подтверждением)
    subscribe_and_confirm(home.path(), "bob", "alice");

    // alice создаёт пост
    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Пост Алисы").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let post_id = open_user_store(home.path(), "alice").list_posts().unwrap()[0]
        .post_id
        .clone();

    // alice комментирует свой пост
    let comment_body = home.path().join("c.txt");
    std::fs::write(&comment_body, "Мой комментарий").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args([
            "comment",
            "new",
            &post_id,
            "--body-file",
            comment_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    // доставляем пост в bob (нужно, чтобы comment был доступен в thread)
    import_all_direct_emails(home.path(), "alice", "alice@example.org", "post_created");
    // доставляем comment в bob
    import_all_direct_emails(home.path(), "alice", "alice@example.org", "comment_created");

    // проверка: bob видит комментарий в thread
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    let assert = lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["thread", &post_id])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("Мой комментарий"),
        "bob должен видеть комментарий alice в thread:\n{stdout}"
    );
}

#[test]
fn bob_comments_alice_post_alice_distributes_to_subscriber() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["init", "--force"])
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");
    setup_user(home.path(), "eve");

    // bob и eve подписываются на alice (с подтверждением)
    subscribe_and_confirm(home.path(), "bob", "alice");
    subscribe_and_confirm(home.path(), "eve", "alice");

    // sanity: после импорта обоих subscription_confirmed
    // в БД alice должно быть 2 подписчика ресурса alice@example.org
    {
        let alice_store = open_user_store(home.path(), "alice");
        let subs = alice_store
            .list_subscriptions_for_resource("alice@example.org")
            .expect("list subscriptions");
        let addrs: Vec<&str> = subs
            .iter()
            .map(|s| s.subscriber_delivery_address.as_str())
            .collect();
        assert!(
            addrs.contains(&"bob@example.org"),
            "alice должна знать bob как подписчика, получили {addrs:?}"
        );
        assert!(
            addrs.contains(&"eve@example.org"),
            "alice должна знать eve как подписчика, получили {addrs:?}"
        );
        assert_eq!(
            subs.len(),
            2,
            "alice должна иметь ровно 2 подписчика, получили {subs:?}"
        );
    }

    // alice создаёт пост
    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Пост Алисы").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let post_id = open_user_store(home.path(), "alice").list_posts().unwrap()[0]
        .post_id
        .clone();

    for to in ["bob", "eve"] {
        let records = outbox_records(home.path(), "alice");
        let post = records
            .iter()
            .find(|r| {
                liveletters_protocol::decode_message(&r.message_body)
                    .ok()
                    .map(|m| m.envelope().event_type() == "post_created")
                    .unwrap_or(false)
            })
            .expect("alice должна положить post_created в outbox");
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["cu", to])
            .assert()
            .success();
        let eml = build_direct_eml("alice@example.org", &format!("{to}@example.org"), post);
        import_eml(home.path(), &eml);
    }

    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    let comment_body = home.path().join("c.txt");
    std::fs::write(&comment_body, "Комментарий Боба").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args([
            "comment",
            "new",
            &post_id,
            "--body-file",
            comment_body.to_str().unwrap(),
        ])
        .assert()
        .success();
    import_all_direct_emails(home.path(), "bob", "bob@example.org", "comment_created");

    // ожидание пересылки: alice должна положить в свой outbox
    // comment_created с Direct([eve]) (bob — автор, исключён)
    let records = outbox_records(home.path(), "alice");
    let redist: Vec<_> = records
        .iter()
        .filter(|r| {
            liveletters_protocol::decode_message(&r.message_body)
                .ok()
                .map(|m| m.envelope().event_type() == "comment_created")
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !redist.is_empty(),
        "alice должна положить в outbox comment_created для пересылки подписчикам"
    );
    for r in &redist {
        match &r.delivery {
            liveletters_store::OutboxDelivery::Direct(addrs) => {
                assert_eq!(
                    addrs,
                    &vec!["eve@example.org".to_owned()],
                    "пересылка должна идти только eve, не bob"
                );
                assert!(
                    !addrs.contains(&"bob@example.org".to_owned()),
                    "bob не должен получать своё же письмо повторно"
                );
            }
            liveletters_store::OutboxDelivery::ResourceSubscribers => {
                panic!(
                    "пересылка должна быть уже разрешена в Direct([eve]), \
                     а не ResourceSubscribers — иначе bob получит своё же письмо"
                );
            }
        }
        // event_type — технический идентификатор
        assert_eq!(
            r.event_type, "comment_created",
            "event_type пересылки должен быть техническим, получили {:?}",
            r.event_type
        );
        // subject — локализованная строка на языке отправителя (alice = ru)
        let subject = r
            .subject
            .as_deref()
            .expect("redistribute должен иметь локализованный subject");
        assert!(
            subject.contains("Новый комментарий"),
            "subject пересылки должен быть локализован, получили {subject:?}"
        );
    }

    // доставляем комментарий alice
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    import_all_direct_emails(home.path(), "bob", "bob@example.org", "comment_created");

    // проверка: alice сохранила комментарий bob в своей БД
    // (потому что alice — источник истины для блога).
    {
        let alice_store = open_user_store(home.path(), "alice");
        let comments = alice_store.list_comments_for_post(&post_id).unwrap();
        assert_eq!(comments.len(), 1, "alice должна сохранить комментарий bob");
        assert_eq!(comments[0].body, "Комментарий Боба");
    }
}

#[test]
fn alice_redistributes_bobs_comment_to_other_subscriber() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["init", "--force"])
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");
    setup_user(home.path(), "eve");

    // bob и eve подписываются на alice (с подтверждением)
    subscribe_and_confirm(home.path(), "bob", "alice");
    subscribe_and_confirm(home.path(), "eve", "alice");

    let body_path = home.path().join("body.txt");
    std::fs::write(&body_path, "Пост Алисы").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["post", "new", "--body-file", body_path.to_str().unwrap()])
        .assert()
        .success();
    let post_id = open_user_store(home.path(), "alice").list_posts().unwrap()[0]
        .post_id
        .clone();

    for to in ["bob", "eve"] {
        let records = outbox_records(home.path(), "alice");
        let post = records
            .iter()
            .find(|r| {
                liveletters_protocol::decode_message(&r.message_body)
                    .ok()
                    .map(|m| m.envelope().event_type() == "post_created")
                    .unwrap_or(false)
            })
            .expect("alice должна положить post_created в outbox");
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["cu", to])
            .assert()
            .success();
        let eml = build_direct_eml("alice@example.org", &format!("{to}@example.org"), post);
        import_eml(home.path(), &eml);
    }

    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    let comment_body = home.path().join("c.txt");
    std::fs::write(&comment_body, "Комментарий Боба").unwrap();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args([
            "comment",
            "new",
            &post_id,
            "--body-file",
            comment_body.to_str().unwrap(),
        ])
        .assert()
        .success();
    import_all_direct_emails(home.path(), "bob", "bob@example.org", "comment_created");

    // ожидание пересылки: alice должна положить в свой outbox
    // comment_created с Direct([eve]) (bob — автор, исключён)
    let records = outbox_records(home.path(), "alice");
    let redist: Vec<_> = records
        .iter()
        .filter(|r| {
            liveletters_protocol::decode_message(&r.message_body)
                .ok()
                .map(|m| m.envelope().event_type() == "comment_created")
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !redist.is_empty(),
        "alice должна положить в outbox comment_created для пересылки подписчикам"
    );
    for r in &redist {
        match &r.delivery {
            liveletters_store::OutboxDelivery::Direct(addrs) => {
                assert_eq!(
                    addrs,
                    &vec!["eve@example.org".to_owned()],
                    "пересылка должна идти только eve, не bob"
                );
                assert!(
                    !addrs.contains(&"bob@example.org".to_owned()),
                    "bob не должен получать своё же письмо повторно"
                );
            }
            liveletters_store::OutboxDelivery::ResourceSubscribers => {
                panic!(
                    "пересылка должна быть уже разрешена в Direct([eve]), \
                     а не ResourceSubscribers — иначе bob получит своё же письмо"
                );
            }
        }
        // event_type — технический идентификатор
        assert_eq!(
            r.event_type, "comment_created",
            "event_type пересылки должен быть техническим, получили {:?}",
            r.event_type
        );
        // subject — локализованная строка на языке отправителя (alice = ru)
        let subject = r
            .subject
            .as_deref()
            .expect("redistribute должен иметь локализованный subject");
        assert!(
            subject.contains("Новый комментарий"),
            "subject пересылки должен быть локализован, получили {subject:?}"
        );
    }
}

// Тихий импорт, чтобы компилятор не ругался на неиспользуемый `DomainEventPayload`
// (он нужен был в старом коде для построения eml, сейчас eml строит
// `liveletters-mime::build_protocol_email`).
#[allow(dead_code)]
fn _ensure_used() {
    let _ = std::mem::size_of::<DomainEventPayload>();
}
