//! Утилиты для тестов команд: создают временный дом, инициализируют
//! секретный ящик и список идентичностей.

use std::path::Path;

use liveletters_config::{IdentityConfig, IdentityMeta, MailSettings};
use liveletters_output::CommandContext;
use liveletters_store::{MailSettingsRecord, Store, UserSettingsRecord};
use tempfile::TempDir;

pub struct TestHome {
    pub dir: TempDir,
}

impl TestHome {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("tempdir");
        Self { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn ctx(&self, identity: &str) -> CommandContext {
        CommandContext {
            home: self.dir.path().to_path_buf(),
            state_home: self.dir.path().to_path_buf(),
            identity_name: identity.to_owned(),
        }
    }

    pub fn add_identity(&self, name: &str) {
        let cfg = sample_identity(name);
        let store = Store::open_for_home_dir(self.dir.path().join("users").join(name)).unwrap();
        store
            .save_user_settings_record(&UserSettingsRecord {
                profile_id: name.to_owned(),
                nickname: cfg.display_name.clone(),
                email_address: cfg.mail.publish.clone(),
                avatar_url: None,
                language: "ru".to_owned(),
                setup_completed: true,
            })
            .unwrap();
        if cfg.mail.smtp().is_some() || cfg.mail.imap().is_some() {
            let smtp = cfg.mail.smtp();
            let imap = cfg.mail.imap();
            store
                .save_mail_settings_record(&MailSettingsRecord {
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
                })
                .unwrap();
        }
        store
            .save_receive_addresses(name, &cfg.mail.receive)
            .unwrap();
        store
            .save_resources_owned(name, &cfg.meta.resources_owned)
            .unwrap();
        store
            .save_local_subscriptions(
                name,
                &cfg.meta
                    .subscriptions
                    .iter()
                    .map(|r| r.as_str().to_owned())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
    }
}

pub fn sample_identity(name: &str) -> IdentityConfig {
    IdentityConfig {
        display_name: format!("Тест {name}"),
        mail: MailSettings {
            publish: format!("https://example.com/{name}/"),
            receive: vec![format!("comments+{name}@example.com")],
            smtp: None,
            imap: None,
        },
        meta: IdentityMeta::default(),
    }
}
