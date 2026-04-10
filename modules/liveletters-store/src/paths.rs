use std::{
    env,
    path::{Path, PathBuf},
};

use crate::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePaths {
    data_dir: PathBuf,
    database_path: PathBuf,
    runtime_log_dir: PathBuf,
}

impl StorePaths {
    pub fn for_home_dir(home_dir: impl AsRef<Path>) -> Self {
        let data_dir = home_dir.as_ref().join(".liveletters");
        let database_path = data_dir.join("liveletters.sqlite3");
        let runtime_log_dir = data_dir.join("runtime-logs");

        Self {
            data_dir,
            database_path,
            runtime_log_dir,
        }
    }

    pub fn from_environment() -> Result<Self, StoreError> {
        let home_dir = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .ok_or(StoreError::MissingHomeDirectory)?;

        Ok(Self::for_home_dir(home_dir))
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn runtime_log_dir(&self) -> &Path {
        &self.runtime_log_dir
    }
}
