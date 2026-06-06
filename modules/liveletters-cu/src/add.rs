use std::path::Path;

use liveletters_config::IdentityConfig;
use liveletters_store::{MailSettingsRecord, Store};

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
    liveletters_config::save_identity(&ctx.home, name, &cfg)?;
    save_mail_settings_from_identity(ctx, name, &cfg)?;
    println!("добавлен identities/{name}.toml");
    Ok(())
}

fn save_mail_settings_from_identity(
    ctx: &liveletters_output::CommandContext,
    name: &str,
    cfg: &IdentityConfig,
) -> Result<(), CuError> {
    if cfg.mail.smtp().is_none() && cfg.mail.imap().is_none() {
        return Ok(());
    }

    let store = Store::open_for_home_dir(ctx.home.join("users").join(name))?;
    let smtp = cfg.mail.smtp();
    let imap = cfg.mail.imap();
    let record = MailSettingsRecord {
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
    };
    store.save_mail_settings_record(&record)?;
    Ok(())
}
