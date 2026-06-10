use assert_cmd::Command;
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
    let to = match &record.delivery {
        liveletters_store::OutboxDelivery::Direct(addrs) => addrs
            .first()
            .cloned()
            .expect("Direct с пустым списком — недопустимо"),
        liveletters_store::OutboxDelivery::ResourceSubscribers => {
            panic!(
                "{envelope_event_type} у {user} имеет ResourceSubscribers, \
                 а ожидался Direct. Тест должен явно проверять delivery."
            );
        }
    };
    let eml = build_direct_eml(from, &to, record);
    // перед импортом переключаем текущего пользователя на адресата письма,
    // иначе inbox import запишет событие в БД не того пользователя
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
    vec![eml]
}

#[test]
fn two_users_subscribe_posts_appear_in_feed() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("init")
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");

    // ── bob подписывается на alice ──
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["sub", "alice@example.org"])
        .assert()
        .success();

    // письмо-подписка из bob → alice (Direct)
    {
        let records = outbox_records(home.path(), "bob");
        let sub = find_outbox(&records, "subscription_changed")
            .expect("bob должен положить subscription_changed в outbox");
        assert_eq!(
            sub.delivery,
            liveletters_store::OutboxDelivery::Direct(vec!["alice@example.org".to_owned()]),
            "subscription_changed от bob должен быть Direct на адрес владельца блога"
        );
    }
    import_all_direct_emails(
        home.path(),
        "bob",
        "bob@example.org",
        "subscription_changed",
    );

    // ── alice создаёт два поста ──
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "alice"])
        .assert()
        .success();
    for (i, body) in ["Первый пост", "Второй пост"].iter().enumerate() {
        let path = home.path().join(format!("body{i}.txt"));
        std::fs::write(&path, body).unwrap();
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["post", "new", "--body-file", path.to_str().unwrap()])
            .assert()
            .success();
    }

    // письма-постов из alice — должны быть адресованы подписчикам (bob)
    {
        let records = outbox_records(home.path(), "alice");
        let posts: Vec<_> = records
            .iter()
            .filter(|r| {
                liveletters_protocol::decode_message(&r.message_body)
                    .ok()
                    .map(|m| m.envelope().event_type() == "post_created")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(
            posts.len(),
            2,
            "alice должна положить 2 post_created в outbox, получили {}",
            posts.len()
        );
        for p in &posts {
            assert_eq!(
                p.delivery,
                liveletters_store::OutboxDelivery::ResourceSubscribers,
                "post_created должен быть адресован подписчикам ресурса"
            );
        }
    }
    let post_emls: Vec<String> = {
        let records = outbox_records(home.path(), "alice");
        records
            .iter()
            .filter(|r| {
                liveletters_protocol::decode_message(&r.message_body)
                    .ok()
                    .map(|m| m.envelope().event_type() == "post_created")
                    .unwrap_or(false)
            })
            .map(|r| {
                let to = match &r.delivery {
                    liveletters_store::OutboxDelivery::Direct(addrs) => addrs[0].clone(),
                    liveletters_store::OutboxDelivery::ResourceSubscribers => {
                        "bob@example.org".to_owned()
                    }
                };
                build_direct_eml("alice@example.org", &to, r)
            })
            .collect()
    };
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    for eml in &post_emls {
        import_eml(home.path(), eml);
    }

    // ── проверка ленты bob ──
    let assert = lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("feed")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(
        stdout.contains("Первый пост"),
        "feed must contain 'Первый пост':\n{stdout}"
    );
    assert!(
        stdout.contains("Второй пост"),
        "feed must contain 'Второй пост':\n{stdout}"
    );
    assert!(
        !stdout.contains("acct_"),
        "acct_* must not appear in feed:\n{stdout}"
    );
    assert!(
        stdout.contains("от alice"),
        "feed must show 'от alice' (nickname):\n{stdout}"
    );
}

#[test]
fn alice_comments_own_post_subscriber_sees_it() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("init")
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");

    // bob подписывается на alice
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["sub", "alice@example.org"])
        .assert()
        .success();

    // доставляем уведомление alice (Direct)
    import_all_direct_emails(
        home.path(),
        "bob",
        "bob@example.org",
        "subscription_changed",
    );

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
            "--post",
            &post_id,
            "--body-file",
            comment_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    // доставляем bob'у сначала сам пост, иначе thread не найдёт post_id
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .args(["cu", "bob"])
        .assert()
        .success();
    {
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
        let eml = build_direct_eml("alice@example.org", "bob@example.org", post);
        import_eml(home.path(), &eml);
    }

    // ожидание: comment_created у alice адресован подписчикам ресурса
    // (частный случай — Alice комментирует свой пост, рассылает всем подписчикам)
    {
        let records = outbox_records(home.path(), "alice");
        let comment = find_outbox(&records, "comment_created")
            .expect("alice должна положить comment_created в outbox");
        assert_eq!(
            comment.delivery,
            liveletters_store::OutboxDelivery::ResourceSubscribers,
            "comment от автора блога должен быть адресован подписчикам ресурса"
        );
        // resource_id должен совпадать с блогом alice
        assert_eq!(comment.resource_id, "alice@example.org");
    }

    // доставляем комментарий bob (как подписчику)
    {
        let records = outbox_records(home.path(), "alice");
        let comment = find_outbox(&records, "comment_created").unwrap();
        let eml = build_direct_eml("alice@example.org", "bob@example.org", comment);
        import_eml(home.path(), &eml);
    }

    // проверка: bob видит комментарий в thread
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
        .arg("init")
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");
    setup_user(home.path(), "eve");

    // bob и eve подписываются на alice
    for sub in ["bob", "eve"] {
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["cu", sub])
            .assert()
            .success();
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["sub", "alice@example.org"])
            .assert()
            .success();
    }

    // доставляем уведомления alice (Direct от bob и от eve)
    import_all_direct_emails(
        home.path(),
        "bob",
        "bob@example.org",
        "subscription_changed",
    );
    import_all_direct_emails(
        home.path(),
        "eve",
        "eve@example.org",
        "subscription_changed",
    );

    // sanity: после импорта обоих subscription_changed
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

    // доставляем пост bob и eve
    {
        let records = outbox_records(home.path(), "alice");
        let posts: Vec<_> = records
            .iter()
            .filter(|r| {
                liveletters_protocol::decode_message(&r.message_body)
                    .ok()
                    .map(|m| m.envelope().event_type() == "post_created")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(posts.len(), 1, "alice должна создать 1 post_created");
        for to in ["bob", "eve"] {
            lltt_cmd()
                .env("LIVELETTERS_HOME", home.path())
                .args(["cu", to])
                .assert()
                .success();
            let eml = build_direct_eml("alice@example.org", &format!("{to}@example.org"), posts[0]);
            import_eml(home.path(), &eml);
        }
    }

    // bob комментирует пост alice
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
            "--post",
            &post_id,
            "--body-file",
            comment_body.to_str().unwrap(),
        ])
        .assert()
        .success();

    // ожидание 1: bob отправляет comment_created прямо alice (Direct)
    {
        let records = outbox_records(home.path(), "bob");
        let comment = find_outbox(&records, "comment_created")
            .expect("bob должен положить comment_created в outbox");
        assert_eq!(
            comment.delivery,
            liveletters_store::OutboxDelivery::Direct(vec!["alice@example.org".to_owned()]),
            "комментарий чужого поста должен уходить владельцу блога, а не подписчикам bob"
        );
        assert_eq!(comment.resource_id, "alice@example.org");
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

/// Сценарий из исходного требования: «После этого Alice смотрит кто на
/// неё подписан кроме Bob и всем им рассылает этот новый комментарий
/// Bob'а».
///
/// На момент написания этот сценарий **не реализован** — Alice при
/// получении `comment_created` от Bob не кладёт автоматически
/// outbox-запись для пересылки остальным подписчикам. Это будет
/// закрыто отдельным планом (см. `.plans/260609-<...>-comment-redistribute.md`).
///
/// Этот тест красный по причине отсутствия реализации пересылки.
#[test]
fn alice_redistributes_bobs_comment_to_other_subscriber() {
    let home = TempDir::new().expect("tempdir");
    lltt_cmd()
        .env("LIVELETTERS_HOME", home.path())
        .arg("init")
        .assert()
        .success();
    setup_user(home.path(), "alice");
    setup_user(home.path(), "bob");
    setup_user(home.path(), "eve");

    for sub in ["bob", "eve"] {
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["cu", sub])
            .assert()
            .success();
        lltt_cmd()
            .env("LIVELETTERS_HOME", home.path())
            .args(["sub", "alice@example.org"])
            .assert()
            .success();
    }
    import_all_direct_emails(
        home.path(),
        "bob",
        "bob@example.org",
        "subscription_changed",
    );
    import_all_direct_emails(
        home.path(),
        "eve",
        "eve@example.org",
        "subscription_changed",
    );

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
            "--post",
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
        // subject пересылки должен быть локализованной строкой,
        // а не техническим идентификатором `comment_created`.
        assert!(
            r.event_type.contains("Новый комментарий"),
            "subject пересылки должен быть локализован через i18n, \
             получили {:?}",
            r.event_type
        );
    }
}
