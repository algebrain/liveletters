use std::path::Path;

use liveletters_config::IdentityConfig;
use liveletters_i18n::detect_system_locale;
use liveletters_store::{MailSettingsRecord, Store, UserSettingsRecord};

use crate::{
    error::CuError,
    name::validate_user_name,
    password_obfuscation::{DialoguerPasswordConfirmer, obfuscate_identity_passwords},
};

pub fn run(
    ctx: &liveletters_output::CommandContext,
    name: &str,
    from: &Path,
) -> Result<(), CuError> {
    validate_user_name(name)?;
    if !from.exists() {
        return Err(CuError::FromFileMissing(from.to_path_buf()));
    }
    let raw = std::fs::read_to_string(from)?;
    let mut cfg: IdentityConfig = toml::from_str(&raw)
        .map_err(|e| CuError::Config(liveletters_config::ConfigError::Toml(e.to_string())))?;
    let mut confirmer = DialoguerPasswordConfirmer;
    let user_state_home = ctx.home.join("users").join(name);
    let changed = obfuscate_identity_passwords(&user_state_home, &mut cfg, &mut confirmer)?;
    if changed {
        let obfuscated = toml::to_string_pretty(&cfg)
            .map_err(|e| CuError::Config(liveletters_config::ConfigError::Toml(e.to_string())))?;
        std::fs::write(from, obfuscated)?;
    }
    let store = Store::open_for_home_dir(ctx.home.join("users").join(name))?;
    save_identity_to_db(&store, name, &cfg)?;
    println!("добавлен {name}");
    Ok(())
}

fn save_identity_to_db(store: &Store, name: &str, cfg: &IdentityConfig) -> Result<(), CuError> {
    store.save_user_settings_record(&UserSettingsRecord {
        profile_id: name.to_owned(),
        nickname: cfg.display_name.clone(),
        email_address: cfg.mail.publish.clone(),
        avatar_url: None,
        language: detect_system_locale().as_str().to_owned(),
        setup_completed: true,
    })?;

    if cfg.mail.smtp().is_some() || cfg.mail.imap().is_some() {
        let smtp = cfg.mail.smtp();
        let imap = cfg.mail.imap();
        store.save_mail_settings_record(&MailSettingsRecord {
            profile_id: name.to_owned(),
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

    store.save_receive_addresses(name, &cfg.mail.receive)?;
    store.save_resources_owned(
        name,
        &cfg.meta
            .resources_owned
            .iter()
            .map(|r| r.as_str().to_owned())
            .collect::<Vec<_>>(),
    )?;
    store.save_local_subscriptions(
        name,
        &cfg.meta
            .subscriptions
            .iter()
            .map(|r| r.as_str().to_owned())
            .collect::<Vec<_>>(),
    )?;
    Ok(())
}
