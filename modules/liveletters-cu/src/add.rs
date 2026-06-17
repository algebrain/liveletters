use std::path::Path;

use liveletters_config::IdentityConfig;
use liveletters_i18n::detect_system_locale;
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

    // Инвариант после `lltt user add`: оба поля в `UserSettingsRecord`
    // (email_address и nickname) непустые. E-mail обязателен —
    // без него нельзя сформировать Message-ID для DSN и домен для envelope.
    if cfg.mail.publish.trim().is_empty() {
        return Err(CuError::InvalidArgs(
            "mail.publish пустой; e-mail обязателен для lltt user add".to_owned(),
        ));
    }
    if !cfg.mail.publish.contains('@') {
        return Err(CuError::InvalidArgs(format!(
            "mail.publish «{}» не содержит @; e-mail обязателен",
            cfg.mail.publish
        )));
    }
    // display_name опционален: если в черновике пусто, берём локальную
    // часть e-mail (до @). Пример: bob@example.com → display_name = "bob".
    if cfg.display_name.trim().is_empty() {
        let local = cfg.mail.publish.split('@').next().unwrap_or("").trim();
        if !local.is_empty() {
            cfg.display_name = local.to_owned();
        }
    }

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
    // Атомарное сохранение: UPSERT в `authors` (email + nickname) +
    // UPSERT в `user_settings` (FK на authors.email). После Этапа 1
    // оба поля непустые (валидируется выше).
    store.save_identity(
        name,
        cfg.mail.publish.as_str(),
        cfg.display_name.as_str(),
        None,
        detect_system_locale().as_str(),
        true,
    )?;

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
            initial_lookback_days: 1,
        })?;
    }

    store.save_receive_addresses(name, &cfg.mail.receive)?;

    // Предзапись внешних адресов в `authors` до того, как таблицы
    // `resources_owned` и `local_subscriptions` начнут на них ссылаться
    // (FK → authors.email). Адрес самого пользователя (`mail.publish`)
    // уже в `authors` (source = "self"), здесь он пропускается.
    let own = cfg.mail.publish.as_str();
    let mut pending: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for r in &cfg.meta.resources_owned {
        let s = r.as_str();
        if s != own {
            pending.insert(s);
        }
    }
    for s in &cfg.meta.subscriptions {
        let addr = s.as_str();
        if addr != own {
            pending.insert(addr);
        } else {
            eprintln!(
                "предупреждение: адрес «{addr}» в meta.subscriptions — это ваш собственный; \
                 нельзя быть подписанным на самого себя, запись пропущена"
            );
        }
    }
    for addr in &pending {
        let nickname = addr.split('@').next().unwrap_or(addr);
        store.save_author(addr, nickname, "origin")?;
    }

    store.save_resources_owned(
        name,
        &cfg.meta
            .resources_owned
            .iter()
            .map(|r| r.as_str().to_owned())
            .collect::<Vec<_>>(),
    )?;
    let subscriptions_filtered: Vec<String> = cfg
        .meta
        .subscriptions
        .iter()
        .map(|r| r.as_str())
        .filter(|r| *r != own)
        .map(str::to_owned)
        .collect();
    store.save_local_subscriptions(name, &subscriptions_filtered)?;
    Ok(())
}
