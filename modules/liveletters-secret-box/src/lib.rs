//! XChaCha20-Poly1305 обфускация секретов на диске.
//!
//! Крейт ответственен за один конкретный механизм: превращение открытого
//! секрета (например, пароля почты) в формат `obf:v1:<base64>`, который
//! можно безопасно хранить в БД. Ключ шифрования лежит отдельным файлом
//! `mail-password-obfuscation.key` в домашнем каталоге и создаётся при
//! первом обращении с правами 0o600 на Unix-системах.

mod codec;
mod error;
mod key_file;

use std::path::{Path, PathBuf};

pub use error::SecretBoxError;

/// Шифр-сейф с привязкой к файлу ключа на диске.
pub struct SecretBox {
    key_path: PathBuf,
    key: [u8; key_file::KEY_LEN],
}

impl std::fmt::Debug for SecretBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretBox")
            .field("key_path", &self.key_path)
            .finish_non_exhaustive()
    }
}

impl SecretBox {
    /// Открывает сейф по существующему файлу ключа. Создавать ключ не пытается:
    /// это разделение ответственности — за создание отвечает `ensure_key`.
    pub fn open(key_path: &Path) -> Result<Self, SecretBoxError> {
        let key = key_file::read_key(key_path)?;
        Ok(Self {
            key_path: key_path.to_path_buf(),
            key,
        })
    }

    /// Создаёт файл ключа (если его нет) и открывает сейф.
    pub fn open_or_create(key_path: &Path) -> Result<Self, SecretBoxError> {
        let key = key_file::ensure_key(key_path)?;
        Ok(Self {
            key_path: key_path.to_path_buf(),
            key,
        })
    }

    pub fn key_path(&self) -> &Path {
        &self.key_path
    }

    pub fn obfuscate(&self, plaintext: &str) -> Result<String, SecretBoxError> {
        codec::encrypt(&self.key, plaintext)
    }

    pub fn deobfuscate(&self, stored: &str) -> Result<String, SecretBoxError> {
        codec::decrypt(&self.key, stored)
    }

    pub fn is_obfuscated(stored: &str) -> bool {
        stored.starts_with(codec::OBFUSCATED_PREFIX)
    }
}

pub fn default_key_path(data_dir: &Path) -> PathBuf {
    key_file::key_path_for(data_dir)
}
