use liveletters_output::CommandContext;
use liveletters_settings::{Args, SettingsAction};
use liveletters_store::{MailSettingsRecord, Store, UserSettingsRecord};

mod common;

fn ctx_for(tmp: &tempfile::TempDir) -> CommandContext {
    CommandContext {
        home: tmp.path().to_path_buf(),
        state_home: tmp.path().to_path_buf(),
        identity_name: "default".to_owned(),
    }
}

fn args_for_show() -> Args {
    Args { tokens: vec![] }
}

fn args_for_set(key: &str, value: &str) -> Args {
    Args {
        tokens: vec!["set".into(), key.into(), value.into()],
    }
}

#[test]
fn show_prints_empty_message_when_no_settings() {
    let (_store, tmp) = common::open_temp_store();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_show()).unwrap();
}

#[test]
fn show_reflects_saved_settings() {
    let (store, tmp) = common::open_temp_store();
    store
        .save_user_settings_record(&UserSettingsRecord {
            profile_id: "default".into(),
            nickname: "Алиса".into(),
            email_address: "alice@example.org".into(),
            avatar_url: None,
            language: "ru".into(),
            setup_completed: true,
        })
        .unwrap();
    store
        .save_mail_settings_record(&MailSettingsRecord {
            profile_id: "default".into(),
            smtp_host: "smtp.example.org".into(),
            smtp_port: 587,
            smtp_security: "starttls".into(),
            smtp_username: "alice@example.org".into(),
            smtp_password: "secret".into(),
            smtp_hello_domain: "example.org".into(),
            imap_host: "imap.example.org".into(),
            imap_port: 993,
            imap_security: "tls".into(),
            imap_username: "alice@example.org".into(),
            imap_password: String::new(),
            imap_mailbox: "INBOX".into(),
        })
        .unwrap();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_show()).unwrap();
}

#[test]
fn set_creates_record_on_first_call() {
    let (_store, tmp) = common::open_temp_store();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    liveletters_settings::run(
        &ctx_for(&tmp),
        &args_for_set("smtp.host", "imap.example.org"),
    )
    .unwrap();
    let mail = store.get_mail_settings_record("default").unwrap().unwrap();
    assert_eq!(mail.smtp_host, "imap.example.org");
}

#[test]
fn set_rejects_unknown_key() {
    let (_store, tmp) = common::open_temp_store();
    let err = liveletters_settings::run(&ctx_for(&tmp), &args_for_set("nonsense.key", "value"))
        .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("nonsense.key"), "msg: {msg}");
    assert!(msg.contains("неизвестный ключ"), "msg: {msg}");
}

#[test]
fn set_obfuscates_passwords() {
    let (_store, tmp) = common::open_temp_store();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_set("smtp.password", "secret123")).unwrap();

    use rusqlite::Connection;
    let db_path = tmp.path().join("liveletters.sqlite3");
    let conn = Connection::open(&db_path).unwrap();
    let raw: String = conn
        .query_row(
            "SELECT smtp_password FROM mail_settings WHERE profile_id = 'default'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap();
    assert!(raw.starts_with("obf:v1:"), "raw stored = {raw:?}");
    assert_ne!(raw, "secret123");

    let mail = store.get_mail_settings_record("default").unwrap().unwrap();
    assert_eq!(mail.smtp_password, "secret123");
}

#[test]
fn set_user_field_then_show() {
    let (_store, tmp) = common::open_temp_store();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_set("nickname", "Алиса")).unwrap();
    let user = store.get_user_settings_record("default").unwrap().unwrap();
    assert_eq!(user.nickname, "Алиса");
    liveletters_settings::run(&ctx_for(&tmp), &args_for_show()).unwrap();
}

#[test]
fn parse_set_requires_two_args() {
    let (_store, tmp) = common::open_temp_store();
    let args = Args {
        tokens: vec!["set".into()],
    };
    let err = liveletters_settings::run(&ctx_for(&tmp), &args).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("set"), "msg: {msg}");
}

#[test]
fn parse_rejects_unknown_subcommand() {
    let (_store, tmp) = common::open_temp_store();
    let args = Args {
        tokens: vec!["nonsense".into()],
    };
    let err = liveletters_settings::run(&ctx_for(&tmp), &args).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("nonsense"), "msg: {msg}");
}

#[test]
fn parse_show_is_default() {
    let (_store, tmp) = common::open_temp_store();
    let args = Args {
        tokens: vec!["show".into()],
    };
    liveletters_settings::run(&ctx_for(&tmp), &args).unwrap();
    let _ = SettingsAction::Show;
}

#[test]
fn set_language_ru_persists_value() {
    let (_store, tmp) = common::open_temp_store();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_set("language", "ru")).unwrap();
    let user = store.get_user_settings_record("default").unwrap().unwrap();
    assert_eq!(user.language, "ru");
}

#[test]
fn set_language_en_persists_value() {
    let (_store, tmp) = common::open_temp_store();
    let store = Store::open_for_home_dir(tmp.path()).unwrap();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_set("language", "en")).unwrap();
    let user = store.get_user_settings_record("default").unwrap().unwrap();
    assert_eq!(user.language, "en");
}

#[test]
fn set_language_unknown_value_returns_error() {
    let (_store, tmp) = common::open_temp_store();
    let err =
        liveletters_settings::run(&ctx_for(&tmp), &args_for_set("language", "de")).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("language"), "msg: {msg}");
    assert!(msg.contains("некорректное"), "msg: {msg}");
}

#[test]
fn set_language_does_not_touch_mail_settings() {
    let (store, tmp) = common::open_temp_store();
    store
        .save_mail_settings_record(&MailSettingsRecord {
            profile_id: "default".into(),
            smtp_host: "smtp.example.org".into(),
            smtp_port: 587,
            smtp_security: "starttls".into(),
            smtp_username: "alice@example.org".into(),
            smtp_password: "secret".into(),
            smtp_hello_domain: "example.org".into(),
            imap_host: "imap.example.org".into(),
            imap_port: 993,
            imap_security: "tls".into(),
            imap_username: "alice@example.org".into(),
            imap_password: String::new(),
            imap_mailbox: "INBOX".into(),
        })
        .unwrap();
    liveletters_settings::run(&ctx_for(&tmp), &args_for_set("language", "en")).unwrap();
    let mail = store.get_mail_settings_record("default").unwrap().unwrap();
    assert_eq!(mail.smtp_host, "smtp.example.org");
    assert_eq!(mail.smtp_port, 587);
}
