use std::path::Path;

use liveletters_config::{load_global, load_identity, save_global};
use liveletters_i18n::{detect_system_locale, parse_locale};
use liveletters_store::{MailSettingsRecord, Store, UserSettingsRecord};

use crate::error::SettingsError;

const ALLOWED_DB_FIELDS: &[&str] = &[
    "nickname",
    "email_address",
    "avatar_url",
    "language",
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
    home: &Path,
    state_home: &Path,
    identity_name: &str,
    key: &str,
    value: &str,
) -> Result<(), SettingsError> {
    if !ALLOWED_DB_FIELDS.contains(&key) {
        return Err(SettingsError::InvalidKey(key.to_owned()));
    }

    if key == "language"
        && let Err(error) = parse_locale(value)
    {
        return Err(SettingsError::InvalidValue(format!("language: {error}")));
    }

    let store = Store::open_for_home_dir(state_home)?;
    ensure_records_exist(&store, home, identity_name, key)?;
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
        run_db_field(home, state_home, identity_name, key, value)
    }
}

fn is_user_field(key: &str) -> bool {
    matches!(
        key,
        "nickname" | "email_address" | "avatar_url" | "language" | "setup_completed"
    )
}

fn ensure_records_exist(
    store: &Store,
    home: &Path,
    profile_id: &str,
    field_key: &str,
) -> Result<(), SettingsError> {
    let identity = load_identity(home, profile_id).ok();
    if store.get_user_settings_record(profile_id)?.is_none() {
        let (nickname, email) = identity
            .as_ref()
            .map(|cfg| (cfg.display_name.clone(), cfg.mail.publish.clone()))
            .unwrap_or_default();
        store.save_user_settings_record(&UserSettingsRecord {
            profile_id: profile_id.into(),
            nickname,
            email_address: email,
            avatar_url: None,
            language: detect_system_locale().as_str().to_owned(),
            setup_completed: false,
        })?;
    }
    if store.get_mail_settings_record(profile_id)?.is_none()
        && (identity
            .as_ref()
            .is_some_and(|cfg| cfg.mail.smtp().is_some() || cfg.mail.imap().is_some())
            || !is_user_field(field_key))
    {
        let smtp = identity.as_ref().and_then(|cfg| cfg.mail.smtp());
        let imap = identity.as_ref().and_then(|cfg| cfg.mail.imap());
        store.save_mail_settings_record(&MailSettingsRecord {
            profile_id: profile_id.into(),
            smtp_host: smtp.map(|s| s.host.clone()).unwrap_or_default(),
            smtp_port: smtp.map(|s| s.port).unwrap_or_default(),
            smtp_security: smtp
                .map(|s| s.security.as_str().to_owned())
                .unwrap_or_else(|| "tls".to_owned()),
            smtp_username: smtp.map(|s| s.username.clone()).unwrap_or_default(),
            smtp_password: smtp.map(|s| s.password.clone()).unwrap_or_default(),
            smtp_hello_domain: smtp.map(|s| s.hello_domain.clone()).unwrap_or_default(),
            imap_host: imap.map(|s| s.host.clone()).unwrap_or_default(),
            imap_port: imap.map(|s| s.port).unwrap_or_default(),
            imap_security: imap
                .map(|s| s.security.as_str().to_owned())
                .unwrap_or_else(|| "tls".to_owned()),
            imap_username: imap.map(|s| s.username.clone()).unwrap_or_default(),
            imap_password: imap.map(|s| s.password.clone()).unwrap_or_default(),
            imap_mailbox: imap
                .map(|s| s.mailbox.clone())
                .unwrap_or_else(|| "INBOX".to_owned()),
        })?;
    }
    Ok(())
}
