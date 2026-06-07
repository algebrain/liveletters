use std::path::Path;

use liveletters_config::{load_global, save_global};
use liveletters_store::{MailSettingsRecord, Store, UserSettingsRecord};

use crate::error::SettingsError;

const ALLOWED_DB_FIELDS: &[&str] = &[
    "nickname",
    "email_address",
    "avatar_url",
    "setup_completed",
    "smtp.host",
    "smtp.port",
    "smtp.security",
    "smtp.username",
    "smtp.password",
    "smtp.hello_domain",
    "imap.host",
    "imap.port",
    "imap.security",
    "imap.username",
    "imap.password",
    "imap.mailbox",
];

const ALLOWED_LOG_FIELDS: &[&str] = &[
    "destination",
    "level",
    "max_size_bytes",
    "keep_files",
    "include_bodies",
];

pub fn run_log_field(home: &Path, field: &str, value: &str) -> Result<(), SettingsError> {
    if !ALLOWED_LOG_FIELDS.contains(&field) {
        return Err(SettingsError::InvalidKey(format!("log.{field}")));
    }
    let mut global = load_global(home)?;
    global
        .log
        .set_field(field, value)
        .map_err(SettingsError::InvalidLogValue)?;
    save_global(home, &global)?;
    println!("настройка обновлена: log.{field}");
    Ok(())
}

pub fn run_db_field(
    state_home: &Path,
    identity_name: &str,
    key: &str,
    value: &str,
) -> Result<(), SettingsError> {
    if !ALLOWED_DB_FIELDS.contains(&key) {
        return Err(SettingsError::InvalidKey(key.to_owned()));
    }

    let store = Store::open_for_home_dir(state_home)?;
    ensure_records_exist(&store, identity_name)?;
    if is_user_field(key) {
        store.update_user_settings_field(identity_name, key, value)?;
    } else {
        store.update_mail_settings_field(identity_name, key, value)?;
    }
    println!("настройка обновлена: {key}");
    Ok(())
}

/// Точка входа для короткой команды `lltt set <ключ> <значение>`.
///
/// Ведёт себя в точности как `lltt settings set <ключ> <значение>`:
/// ключи `log.*` маршрутизируются в глобальный конфиг (`<home>`), остальные —
/// в базу данных текущего пользователя (`<state_home>`).
pub fn run_directly(
    home: &Path,
    state_home: &Path,
    identity_name: &str,
    key: &str,
    value: &str,
) -> Result<(), SettingsError> {
    if let Some(field) = key.strip_prefix("log.") {
        run_log_field(home, field, value)
    } else {
        run_db_field(state_home, identity_name, key, value)
    }
}

fn is_user_field(key: &str) -> bool {
    matches!(
        key,
        "nickname" | "email_address" | "avatar_url" | "setup_completed"
    )
}

fn ensure_records_exist(store: &Store, profile_id: &str) -> Result<(), SettingsError> {
    if store.get_user_settings_record(profile_id)?.is_none() {
        store.save_user_settings_record(&UserSettingsRecord {
            profile_id: profile_id.into(),
            nickname: String::new(),
            email_address: String::new(),
            avatar_url: None,
            setup_completed: false,
        })?;
    }
    if store.get_mail_settings_record(profile_id)?.is_none() {
        store.save_mail_settings_record(&MailSettingsRecord {
            profile_id: profile_id.into(),
            smtp_host: String::new(),
            smtp_port: 0,
            smtp_security: "tls".into(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_hello_domain: String::new(),
            imap_host: String::new(),
            imap_port: 0,
            imap_security: "tls".into(),
            imap_username: String::new(),
            imap_password: String::new(),
            imap_mailbox: "INBOX".into(),
        })?;
    }
    Ok(())
}
