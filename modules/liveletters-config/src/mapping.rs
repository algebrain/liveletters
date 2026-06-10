use liveletters_app_core::AppSettings;

use crate::{IdentityConfig, ImapSettings, MailSecurity, SmtpSettings};

/// Преобразует `IdentityConfig` (диск) в `AppSettings` (память).
/// Поля без значений заполняются дефолтами из `AppSettings::empty()`.
pub fn map_identity_to_settings(identity: &IdentityConfig) -> AppSettings {
    let mut settings = AppSettings::empty();
    settings.nickname = identity.display_name.clone();
    settings.email_address = identity.mail.publish.clone();
    settings.setup_completed = true;

    if let Some(smtp) = identity.mail.smtp() {
        settings.smtp_host = smtp.host.clone();
        settings.smtp_port = smtp.port;
        settings.smtp_security = smtp.security.as_str().to_owned();
        settings.smtp_username = smtp.username.clone();
        settings.smtp_password = smtp.password.clone();
    }

    if let Some(imap) = identity.mail.imap() {
        settings.imap_host = imap.host.clone();
        settings.imap_port = imap.port;
        settings.imap_security = imap.security.as_str().to_owned();
        settings.imap_username = imap.username.clone();
        settings.imap_password = imap.password.clone();
        settings.imap_mailbox = imap.mailbox.clone();
    }

    settings
}

/// Преобразует `AppSettings` (память) в `IdentityConfig` (диск).
/// Используется при обратной записи после редактирования через CLI.
pub fn settings_to_identity(settings: &AppSettings) -> IdentityConfig {
    let smtp = if settings.smtp_host.is_empty() {
        None
    } else {
        Some(SmtpSettings {
            host: settings.smtp_host.clone(),
            port: settings.smtp_port,
            security: parse_mail_security(&settings.smtp_security),
            username: settings.smtp_username.clone(),
            password: settings.smtp_password.clone(),
            pwd_obfuscate: true,
            hello_domain: String::new(),
        })
    };

    let imap = if settings.imap_host.is_empty() {
        None
    } else {
        Some(ImapSettings {
            host: settings.imap_host.clone(),
            port: settings.imap_port,
            security: parse_mail_security(&settings.imap_security),
            username: settings.imap_username.clone(),
            password: settings.imap_password.clone(),
            pwd_obfuscate: true,
            mailbox: settings.imap_mailbox.clone(),
        })
    };

    IdentityConfig {
        display_name: settings.nickname.clone(),
        mail: crate::MailSettings {
            publish: settings.email_address.clone(),
            receive: Vec::new(),
            smtp,
            imap,
        },
        meta: Default::default(),
    }
}

fn parse_mail_security(value: &str) -> MailSecurity {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => MailSecurity::None,
        "tls" | "ssl" | "ssl/tls" => MailSecurity::Tls,
        _ => MailSecurity::StartTls,
    }
}
