use std::{
    fs,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};

use crate::StoreError;

const OBFUSCATED_PREFIX: &str = "obf:v1:";
const KEY_FILENAME: &str = "mail-password-obfuscation.key";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;

pub struct PasswordObfuscator {
    key_path: PathBuf,
    key: [u8; KEY_LEN],
}

impl PasswordObfuscator {
    pub fn load(data_dir: &Path) -> Result<Self, StoreError> {
        let key_path = data_dir.join(KEY_FILENAME);
        let bytes = fs::read(&key_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                StoreError::ProtectedSecretUnavailable {
                    message: format!(
                        "password obfuscation key is missing: {}",
                        key_path.display()
                    ),
                }
            } else {
                error.into()
            }
        })?;

        if bytes.len() != KEY_LEN {
            return Err(StoreError::ProtectedSecretUnavailable {
                message: format!(
                    "password obfuscation key has invalid length: {}",
                    key_path.display()
                ),
            });
        }

        let mut key = [0_u8; KEY_LEN];
        key.copy_from_slice(&bytes);

        Ok(Self { key_path, key })
    }

    pub fn load_or_create(data_dir: &Path) -> Result<Self, StoreError> {
        let key_path = data_dir.join(KEY_FILENAME);
        if key_path.exists() {
            return Self::load(data_dir);
        }

        fs::create_dir_all(data_dir)?;

        let mut key = [0_u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);
        fs::write(&key_path, key)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(Self { key_path, key })
    }

    pub fn obfuscate(&self, plaintext: &str) -> Result<String, StoreError> {
        let cipher = XChaCha20Poly1305::new((&self.key).into());
        let mut nonce = [0_u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);

        let ciphertext = cipher
            .encrypt(XNonce::from_slice(&nonce), plaintext.as_bytes())
            .map_err(|error| StoreError::InvalidProtectedSecretFormat {
                message: format!("failed to obfuscate password: {error}"),
            })?;

        let mut payload = nonce.to_vec();
        payload.extend_from_slice(&ciphertext);

        Ok(format!("{OBFUSCATED_PREFIX}{}", STANDARD.encode(payload)))
    }

    pub fn reveal(&self, stored: &str) -> Result<String, StoreError> {
        let Some(encoded) = stored.strip_prefix(OBFUSCATED_PREFIX) else {
            return Err(StoreError::InvalidProtectedSecretFormat {
                message: "stored secret does not use obf:v1 format".into(),
            });
        };

        let payload = STANDARD.decode(encoded).map_err(|error| {
            StoreError::InvalidProtectedSecretFormat {
                message: format!("protected secret is not valid base64: {error}"),
            }
        })?;

        if payload.len() <= NONCE_LEN {
            return Err(StoreError::InvalidProtectedSecretFormat {
                message: "protected secret payload is too short".into(),
            });
        }

        let (nonce, ciphertext) = payload.split_at(NONCE_LEN);
        let cipher = XChaCha20Poly1305::new((&self.key).into());
        let plaintext = cipher
            .decrypt(XNonce::from_slice(nonce), ciphertext)
            .map_err(|error| StoreError::ProtectedSecretUnavailable {
                message: format!(
                    "failed to reveal protected password using key {}: {error}",
                    self.key_path.display()
                ),
            })?;

        String::from_utf8(plaintext).map_err(|error| StoreError::InvalidProtectedSecretFormat {
            message: format!("protected secret is not valid utf-8: {error}"),
        })
    }

    pub fn is_obfuscated(stored: &str) -> bool {
        stored.starts_with(OBFUSCATED_PREFIX)
    }
}
