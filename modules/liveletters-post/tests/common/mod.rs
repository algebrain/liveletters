//! Утилиты для тестов `lltt post`.

use std::path::Path;

use liveletters_config::{IdentityConfig, IdentityMeta, MailSettings, save_identity};
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
    }

    pub fn open_store(&self) -> Store {
        Store::open_for_home_dir(self.dir.path()).expect("store opens in temp home")
    }
}

pub fn sample_identity(name: &str) -> IdentityConfig {
    IdentityConfig {
        account_id: name.to_owned(),
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
