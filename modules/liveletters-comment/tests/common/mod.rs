//! Утилиты для тестов `lltt comment`.

use std::path::Path;

use liveletters_config::{IdentityConfig, IdentityMeta, MailSettings, save_identity};
use liveletters_i18n::detect_system_locale;
use liveletters_output::CommandContext;
use liveletters_store::Store;
use tempfile::TempDir;

pub struct TestHome {
    pub dir: TempDir,
}

impl TestHome {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("identities")).expect("identities dir");
        let store = Store::open_for_home_dir(dir.path()).expect("store opens");
        store
            .save_identity(
                "default",
                "test@example.test",
                "test",
                None,
                detect_system_locale().as_str(),
                true,
            )
            .expect("save default identity");
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
        save_identity(self.dir.path(), name, &cfg).expect("save identity");
        let store = Store::open_for_home_dir(self.dir.path()).expect("store opens");
        store
            .save_identity(
                name,
                cfg.mail.publish.as_str(),
                cfg.display_name.as_str(),
                None,
                detect_system_locale().as_str(),
                true,
            )
            .expect("save identity record");
    }

    pub fn open_store(&self) -> Store {
        Store::open_for_home_dir(self.dir.path()).expect("store opens in temp home")
    }
}

pub fn sample_identity(name: &str) -> IdentityConfig {
    IdentityConfig {
        display_name: format!("Тест {name}"),
        mail: MailSettings {
            publish: format!("{name}-publish@example.org"),
            receive: vec![format!("{name}-feed@example.org")],
            smtp: None,
            imap: None,
        },
        meta: IdentityMeta::default(),
    }
}
