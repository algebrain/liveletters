//! Утилиты для тестов `lltt thread`.

use liveletters_output::CommandContext;
use liveletters_store::Store;
use tempfile::TempDir;

pub struct TestHome {
    pub dir: TempDir,
}

impl TestHome {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("tempdir");
        Self { dir }
    }

    pub fn init(&self) {
        let store = Store::open_for_home_dir(self.dir.path()).expect("store opens");
        store
            .save_identity("alice", "alice@example.test", "alice", None, "ru", true)
            .expect("save identity");
    }

    pub fn ctx(&self, identity: &str) -> CommandContext {
        CommandContext {
            home: self.dir.path().to_path_buf(),
            state_home: self.dir.path().to_path_buf(),
            identity_name: identity.to_owned(),
        }
    }

    pub fn open_store(&self) -> Store {
        Store::open_for_home_dir(self.dir.path()).expect("store opens in temp home")
    }
}
