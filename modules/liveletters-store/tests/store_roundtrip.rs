use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use liveletters_store::{
    CommentRecord, DeferredEventRecord, MailSettingsRecord, OutboxDelivery, OutboxRecord,
    PostRecord, RawEventRecord, RawMessageRecord, Store, StorePaths,
};
use rusqlite::Connection;

mod common;

fn temp_home_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("liveletters-home-{unique}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn load_raw_mail_passwords(database_path: &std::path::Path) -> (String, String) {
    let connection = Connection::open(database_path).unwrap();
    connection
        .query_row(
            r#"
            SELECT smtp_password, imap_password
            FROM mail_settings
            WHERE profile_id = 'default'
            "#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
}

#[test]
fn in_memory_store_starts_with_empty_state() {
    let (store, _tmp) = common::open_temp_store();

    assert!(store.list_posts().unwrap().is_empty());
    assert!(store.list_comments_for_post("post-1").unwrap().is_empty());
    assert!(store.list_outbox_records().unwrap().is_empty());
    assert!(store.list_raw_message_records().unwrap().is_empty());
    assert!(store.list_raw_event_records().unwrap().is_empty());
    assert!(store.list_deferred_event_records().unwrap().is_empty());
    assert!(store.get_user_settings_record("default").unwrap().is_none());
    assert!(store.get_mail_settings_record("default").unwrap().is_none());
}

#[test]
fn saved_post_can_be_read_back() {
    let (store, _tmp) = common::open_temp_store();

    common::ensure_author(&store, "blog-1", "blog");
    common::ensure_author(&store, "alice", "alice");

    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice".into(),
            created_at: 1_710_000_000,
            body: "Первая запись".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    let posts = store.list_posts().unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].post_id, "post-1");
    assert_eq!(posts[0].body, "Первая запись");
    assert_eq!(posts[0].visibility, "public");
}

#[test]
fn list_posts_returns_newest_first() {
    let (store, _tmp) = common::open_temp_store();

    for (post_id, created_at) in [("old", 1_710_000_000), ("new", 1_710_000_100)] {
        common::ensure_author(&store, "blog-1", "blog");
        common::ensure_author(&store, "alice", "alice");

        store
            .save_post_record(&PostRecord {
                post_id: post_id.into(),
                resource_email: "blog-1".into(),
                author_email: "alice".into(),
                created_at,
                body: post_id.into(),
                visibility: "public".into(),
                hidden: false,
            })
            .unwrap();
    }

    let posts = store.list_posts().unwrap();

    assert_eq!(
        posts
            .iter()
            .map(|post| post.post_id.as_str())
            .collect::<Vec<_>>(),
        vec!["new", "old"]
    );
}

#[test]
fn saved_comment_is_returned_for_its_post() {
    let (store, _tmp) = common::open_temp_store();

    common::ensure_author(&store, "blog-1", "blog");
    common::ensure_author(&store, "alice", "alice");

    store
        .save_post_record(&PostRecord {
            post_id: "post-1".into(),
            resource_email: "blog-1".into(),
            author_email: "alice".into(),
            created_at: 1_710_000_000,
            body: "Первая запись".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    common::ensure_author(&store, "alice", "alice");

    store
        .save_comment_record(&CommentRecord {
            comment_id: "comment-root".into(),
            post_id: "post-1".into(),
            parent_comment_id: None,
            author_email: "alice".into(),
            created_at: 1_710_000_050,
            body: "Корневой".into(),
            visibility: "public".into(),
            hidden: false,
        })
        .unwrap();

    store
        .save_comment_record(&CommentRecord {
            comment_id: "comment-1".into(),
            post_id: "post-1".into(),
            parent_comment_id: Some("comment-root".into()),
            author_email: "alice".into(),
            created_at: 1_710_000_100,
            body: "Ответ".into(),
            visibility: "friends_only".into(),
            hidden: false,
        })
        .unwrap();

    let comments = store.list_comments_for_post("post-1").unwrap();

    assert_eq!(comments.len(), 2);
    let child = comments
        .iter()
        .find(|comment| comment.comment_id == "comment-1")
        .expect("child comment must be listed");
    assert_eq!(child.parent_comment_id.as_deref(), Some("comment-root"));
    assert_eq!(child.body, "Ответ");
}

#[test]
fn store_paths_treat_home_dir_as_data_dir() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    assert_eq!(paths.data_dir(), home_dir);
    assert_eq!(paths.database_path(), home_dir.join("liveletters.sqlite3"));
    assert_eq!(paths.runtime_log_dir(), home_dir.join("runtime-logs"));

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_can_open_for_home_dir_and_create_missing_home_tree() {
    let home_dir = std::env::temp_dir().join(format!(
        "liveletters-missing-home-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    assert!(!home_dir.exists());

    {
        let store = Store::open_for_home_dir(&home_dir).unwrap();
        common::ensure_author(&store, "blog-1", "blog");
        common::ensure_author(&store, "alice", "alice");

        store
            .save_post_record(&PostRecord {
                post_id: "post-1".into(),
                resource_email: "blog-1".into(),
                author_email: "alice".into(),
                created_at: 1_710_000_000,
                body: "Первая запись".into(),
                visibility: "public".into(),
                hidden: false,
            })
            .unwrap();
    }

    let paths = StorePaths::for_home_dir(&home_dir);
    assert!(home_dir.exists());
    assert!(paths.data_dir().exists());
    assert!(paths.database_path().exists());

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_persists_records_under_temp_home() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_at(paths.database_path()).unwrap();
        common::ensure_author(&store, "blog-1", "blog");
        common::ensure_author(&store, "alice", "alice");

        store
            .save_post_record(&PostRecord {
                post_id: "post-1".into(),
                resource_email: "blog-1".into(),
                author_email: "alice".into(),
                created_at: 1_710_000_000,
                body: "Первая запись".into(),
                visibility: "public".into(),
                hidden: false,
            })
            .unwrap();
    }

    assert!(paths.database_path().exists());

    let reopened = Store::open_at(paths.database_path()).unwrap();
    let posts = reopened.list_posts().unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].post_id, "post-1");

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn outbox_records_can_be_saved_and_listed() {
    let (store, _tmp) = common::open_temp_store();
    common::ensure_author(&store, "blog-1", "blog");
    common::ensure_author(&store, "alice@example.org", "alice");

    store
        .save_outbox_record(&OutboxRecord {
            event_id: "event-1".into(),
            event_type: "post_created".into(),
            author_email: "alice@example.org".into(),
            resource_email: Some("blog-1".into()),
            delivery: OutboxDelivery::ResourceSubscribers,
            message_body: "{\"kind\":\"post_created\"}".into(),
            message_id: None,
            subject: None,
            human_readable_body: None,
        })
        .unwrap();

    let outbox = store.list_outbox_records().unwrap();

    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_id, "event-1");
    assert_eq!(outbox[0].event_type, "post_created");
    assert_eq!(outbox[0].delivery, OutboxDelivery::ResourceSubscribers);
}

#[test]
fn raw_message_and_event_journals_can_be_saved() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_raw_message_record(&RawMessageRecord {
            message_id: "message-1".into(),
            raw_message: "raw email".into(),
            status: "applied".into(),
        })
        .unwrap();
    store
        .save_raw_event_record(&RawEventRecord {
            event_id: "event-1".into(),
            event_type: "post_created".into(),
            resource_id: "blog-1".into(),
            payload_json: "{\"kind\":\"post_created\"}".into(),
            apply_status: "applied".into(),
            failure_reason: None,
        })
        .unwrap();

    assert_eq!(store.list_raw_message_records().unwrap().len(), 1);
    assert_eq!(store.list_raw_event_records().unwrap().len(), 1);
    assert!(store.has_raw_event("event-1").unwrap());
    assert_eq!(
        store.list_raw_event_records().unwrap()[0].apply_status,
        "applied"
    );
}

#[test]
fn deferred_events_can_be_saved_and_listed() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "event-2".into(),
            event_type: "comment_created".into(),
            reason: "missing_post".into(),
            payload_json: "{\"kind\":\"comment_created\"}".into(),
        })
        .unwrap();

    let deferred = store.list_deferred_event_records().unwrap();

    assert_eq!(deferred.len(), 1);
    assert_eq!(deferred[0].reason, "missing_post");
}

#[test]
fn deferred_event_can_be_deleted_after_reprocessing() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_deferred_event_record(&DeferredEventRecord {
            event_id: "event-2".into(),
            event_type: "comment_created".into(),
            reason: "missing_post".into(),
            payload_json: "{\"kind\":\"comment_created\"}".into(),
        })
        .unwrap();

    store.delete_deferred_event_record("event-2").unwrap();

    assert!(store.list_deferred_event_records().unwrap().is_empty());
}

#[test]
fn user_and_mail_settings_can_be_saved_and_read_back() {
    let (store, _tmp) = common::open_temp_store();

    store
        .save_identity(
            "default",
            "alice@example.com",
            "alice",
            Some("https://example.com/avatar.png"),
            "ru",
            true,
        )
        .unwrap();

    store
        .save_mail_settings_record(&MailSettingsRecord {
            profile_id: "default".into(),
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            smtp_security: "starttls".into(),
            smtp_username: "alice".into(),
            smtp_password: "secret".into(),
            smtp_hello_domain: "example.com".into(),
            imap_host: "imap.example.com".into(),
            imap_port: 143,
            imap_security: "tls".into(),
            imap_username: "alice".into(),
            imap_password: "secret".into(),
            imap_mailbox: "INBOX".into(),
            initial_lookback_days: 1,
        })
        .unwrap();

    let user = store.get_user_settings_record("default").unwrap().unwrap();
    let mail = store.get_mail_settings_record("default").unwrap().unwrap();

    // Проверка roundtrip initial_lookback_days
    assert_eq!(
        mail.initial_lookback_days, 1,
        "default initial_lookback_days должно быть 1"
    );

    // Обновим до 7 и проверим, что сохранилось
    let mut updated = mail.clone();
    updated.initial_lookback_days = 7;
    store.save_mail_settings_record(&updated).unwrap();
    let got = store.get_mail_settings_record("default").unwrap().unwrap();
    assert_eq!(got.initial_lookback_days, 7);

    // Ник берём из authors.
    let author = store.get_author(&user.author_email).unwrap().unwrap();
    assert_eq!(author.nickname, "alice");
    assert_eq!(user.author_email, "alice@example.com");
    assert!(user.setup_completed);
    assert_eq!(mail.smtp_host, "smtp.example.com");
    assert_eq!(mail.smtp_port, 587);
    assert_eq!(mail.smtp_security, "starttls");
    assert_eq!(mail.imap_mailbox, "INBOX");
    assert_eq!(mail.imap_security, "tls");
}

#[test]
fn file_store_persists_user_and_mail_settings_under_temp_home() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_at(paths.database_path()).unwrap();
        store
            .save_identity("default", "alice@example.com", "alice", None, "ru", true)
            .unwrap();
        store
            .save_mail_settings_record(&MailSettingsRecord {
                profile_id: "default".into(),
                smtp_host: "smtp.example.com".into(),
                smtp_port: 587,
                smtp_security: "starttls".into(),
                smtp_username: "alice".into(),
                smtp_password: "secret".into(),
                smtp_hello_domain: "example.com".into(),
                imap_host: "imap.example.com".into(),
                imap_port: 143,
                imap_security: "starttls".into(),
                imap_username: "alice".into(),
                imap_password: "secret".into(),
                imap_mailbox: "INBOX".into(),
                initial_lookback_days: 1,
            })
            .unwrap();
    }

    let reopened = Store::open_at(paths.database_path()).unwrap();
    let user = reopened
        .get_user_settings_record("default")
        .unwrap()
        .unwrap();
    let mail = reopened
        .get_mail_settings_record("default")
        .unwrap()
        .unwrap();

    // Ник берём из authors.
    let author = reopened.get_author(&user.author_email).unwrap().unwrap();
    assert_eq!(author.nickname, "alice");
    assert!(user.setup_completed);
    assert_eq!(mail.smtp_username, "alice");
    assert_eq!(mail.imap_security, "starttls");

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_obfuscates_passwords_before_persisting_to_sqlite() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_for_home_dir(&home_dir).unwrap();
        store
            .save_mail_settings_record(&MailSettingsRecord {
                profile_id: "default".into(),
                smtp_host: "smtp.example.com".into(),
                smtp_port: 587,
                smtp_security: "starttls".into(),
                smtp_username: "alice".into(),
                smtp_password: "secret".into(),
                smtp_hello_domain: "example.com".into(),
                imap_host: "imap.example.com".into(),
                imap_port: 143,
                imap_security: "starttls".into(),
                imap_username: "alice".into(),
                imap_password: "secret".into(),
                imap_mailbox: "INBOX".into(),
                initial_lookback_days: 1,
            })
            .unwrap();
    }

    let (smtp_password, imap_password) = load_raw_mail_passwords(paths.database_path());

    assert_ne!(smtp_password, "secret");
    assert_ne!(imap_password, "secret");
    assert!(smtp_password.starts_with("obf:v1:"));
    assert!(imap_password.starts_with("obf:v1:"));
    assert!(paths.password_obfuscation_key_path().exists());

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_keeps_empty_passwords_empty_without_creating_key_file() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_for_home_dir(&home_dir).unwrap();
        store
            .save_mail_settings_record(&MailSettingsRecord {
                profile_id: "default".into(),
                smtp_host: "smtp.example.com".into(),
                smtp_port: 587,
                smtp_security: "starttls".into(),
                smtp_username: "alice".into(),
                smtp_password: "".into(),
                smtp_hello_domain: "example.com".into(),
                imap_host: "imap.example.com".into(),
                imap_port: 143,
                imap_security: "starttls".into(),
                imap_username: "alice".into(),
                imap_password: "".into(),
                imap_mailbox: "INBOX".into(),
                initial_lookback_days: 1,
            })
            .unwrap();
    }

    let (smtp_password, imap_password) = load_raw_mail_passwords(paths.database_path());
    let store = Store::open_for_home_dir(&home_dir).unwrap();
    let mail = store.get_mail_settings_record("default").unwrap().unwrap();

    assert_eq!(smtp_password, "");
    assert_eq!(imap_password, "");
    assert_eq!(mail.smtp_password, "");
    assert_eq!(mail.imap_password, "");
    assert!(!paths.password_obfuscation_key_path().exists());

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_lazily_migrates_plaintext_passwords_on_read() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_for_home_dir(&home_dir).unwrap();
        store
            .save_mail_settings_record(&MailSettingsRecord {
                profile_id: "default".into(),
                smtp_host: "smtp.example.com".into(),
                smtp_port: 587,
                smtp_security: "starttls".into(),
                smtp_username: "alice".into(),
                smtp_password: "secret".into(),
                smtp_hello_domain: "example.com".into(),
                imap_host: "imap.example.com".into(),
                imap_port: 143,
                imap_security: "starttls".into(),
                imap_username: "alice".into(),
                imap_password: "secret".into(),
                imap_mailbox: "INBOX".into(),
                initial_lookback_days: 1,
            })
            .unwrap();
    }

    {
        let connection = Connection::open(paths.database_path()).unwrap();
        connection
            .execute(
                r#"
                UPDATE mail_settings
                SET smtp_password = 'legacy-secret',
                    imap_password = 'legacy-secret'
                WHERE profile_id = 'default'
                "#,
                [],
            )
            .unwrap();
    }

    let store = Store::open_for_home_dir(&home_dir).unwrap();
    let mail = store.get_mail_settings_record("default").unwrap().unwrap();
    let (smtp_password, imap_password) = load_raw_mail_passwords(paths.database_path());

    assert_eq!(mail.smtp_password, "legacy-secret");
    assert_eq!(mail.imap_password, "legacy-secret");
    assert!(smtp_password.starts_with("obf:v1:"));
    assert!(imap_password.starts_with("obf:v1:"));
    assert!(paths.password_obfuscation_key_path().exists());

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn file_store_reports_error_when_obfuscated_password_cannot_be_recovered() {
    let home_dir = temp_home_dir();
    let paths = StorePaths::for_home_dir(&home_dir);

    {
        let store = Store::open_for_home_dir(&home_dir).unwrap();
        store
            .save_mail_settings_record(&MailSettingsRecord {
                profile_id: "default".into(),
                smtp_host: "smtp.example.com".into(),
                smtp_port: 587,
                smtp_security: "starttls".into(),
                smtp_username: "alice".into(),
                smtp_password: "secret".into(),
                smtp_hello_domain: "example.com".into(),
                imap_host: "imap.example.com".into(),
                imap_port: 143,
                imap_security: "starttls".into(),
                imap_username: "alice".into(),
                imap_password: "secret".into(),
                imap_mailbox: "INBOX".into(),
                initial_lookback_days: 1,
            })
            .unwrap();
    }

    fs::remove_file(paths.password_obfuscation_key_path()).unwrap();

    let store = Store::open_for_home_dir(&home_dir).unwrap();
    let error = store.get_mail_settings_record("default").unwrap_err();

    assert!(matches!(
        error,
        liveletters_store::StoreError::ProtectedSecretUnavailable { .. }
    ));

    fs::remove_dir_all(home_dir).unwrap();
}

#[test]
fn mail_settings_default_initial_lookback_days_is_one() {
    use liveletters_store::MailSettingsRecord;
    let (_store, _tmp) = common::open_temp_store();
    let default = MailSettingsRecord {
        profile_id: "default".into(),
        ..Default::default()
    };
    assert_eq!(default.initial_lookback_days, 1);
}
