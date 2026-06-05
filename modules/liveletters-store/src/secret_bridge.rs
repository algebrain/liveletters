use std::path::Path;

use liveletters_secret_box::{SecretBox, SecretBoxError};

use crate::StoreError;

fn map(error: SecretBoxError) -> StoreError {
    match error {
        SecretBoxError::Io { source, .. } => StoreError::Io(source),
        SecretBoxError::InvalidKeyLength { .. } => StoreError::ProtectedSecretUnavailable {
            message: format!("{error}"),
        },
        SecretBoxError::InvalidFormat { .. } | SecretBoxError::Crypto { .. } => {
            StoreError::InvalidProtectedSecretFormat {
                message: format!("{error}"),
            }
        }
    }
}

/// Открывает `SecretBox` по существующему ключу. Возвращает `None`, если
/// файла ключа нет — в этом случае обфускация не активна.
pub fn try_load(key_path: &Path) -> Result<Option<SecretBox>, StoreError> {
    if !key_path.exists() {
        return Ok(None);
    }
    SecretBox::open(key_path).map(Some).map_err(map)
}

/// Создаёт ключ, если его нет, и возвращает `SecretBox`. Это путь,
/// которым `Store` обфусцирует новые секреты.
pub fn load_or_create(key_path: &Path) -> Result<SecretBox, StoreError> {
    SecretBox::open_or_create(key_path).map_err(map)
}

/// Шифрует `plaintext` ключом из `key_path`, при необходимости создавая
/// ключ. Все ошибки `SecretBox` транслируются в `StoreError`.
pub fn obfuscate(key_path: &Path, plaintext: &str) -> Result<String, StoreError> {
    let box_ = load_or_create(key_path)?;
    box_.obfuscate(plaintext).map_err(map)
}

/// Расшифровывает `stored` ключом из `key_path`. Если файл ключа
/// отсутствует, возвращает `ProtectedSecretUnavailable`.
pub fn deobfuscate(key_path: &Path, stored: &str) -> Result<String, StoreError> {
    let box_ = try_load(key_path)?.ok_or_else(|| StoreError::ProtectedSecretUnavailable {
        message: format!(
            "key file is missing at {}: cannot reveal obfuscated secret",
            key_path.display()
        ),
    })?;
    box_.deobfuscate(stored).map_err(map)
}
