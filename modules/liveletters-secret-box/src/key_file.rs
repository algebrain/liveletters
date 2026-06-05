use std::{
    fs,
    path::{Path, PathBuf},
};

use chacha20poly1305::aead::{OsRng, rand_core::RngCore};

use crate::error::SecretBoxError;

pub const KEY_LEN: usize = 32;

/// Читает ключ из файла. Возвращает `Io` ошибку, если файл отсутствует.
pub fn read_key(key_path: &Path) -> Result<[u8; KEY_LEN], SecretBoxError> {
    let bytes = fs::read(key_path).map_err(|error| {
        SecretBoxError::io(
            error,
            format!("cannot read key file {}", key_path.display()),
        )
    })?;

    if bytes.len() != KEY_LEN {
        return Err(SecretBoxError::InvalidKeyLength {
            path: key_path.to_path_buf(),
            expected: KEY_LEN,
            actual: bytes.len(),
        });
    }

    let mut key = [0_u8; KEY_LEN];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Создаёт файл ключа, если его нет, и возвращает ключ.
/// На Unix выставляет права 0o600.
pub fn ensure_key(key_path: &Path) -> Result<[u8; KEY_LEN], SecretBoxError> {
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if key_path.exists() {
        return read_key(key_path);
    }

    let mut key = [0_u8; KEY_LEN];
    OsRng.fill_bytes(&mut key);
    fs::write(key_path, key)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(key_path, permissions).map_err(|error| {
            SecretBoxError::io(
                error,
                format!("cannot set 0o600 on key file {}", key_path.display()),
            )
        })?;
    }

    Ok(key)
}

pub fn key_path_for(data_dir: &Path) -> PathBuf {
    data_dir.join("mail-password-obfuscation.key")
}
