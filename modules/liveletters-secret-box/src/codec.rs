use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};

use crate::error::SecretBoxError;

pub(crate) const OBFUSCATED_PREFIX: &str = "obf:v1:";
pub(crate) const NONCE_LEN: usize = 24;

/// Шифрует `plaintext` в формат `obf:v1:<base64(nonce || ciphertext)>`.
pub(crate) fn encrypt(key: &[u8; 32], plaintext: &str) -> Result<String, SecretBoxError> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut nonce = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|error| SecretBoxError::Crypto {
            message: format!("encryption failed: {error}"),
        })?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);

    Ok(format!("{OBFUSCATED_PREFIX}{}", STANDARD.encode(payload)))
}

/// Расшифровывает токен формата `obf:v1:<base64(nonce || ciphertext)>`.
pub(crate) fn decrypt(key: &[u8; 32], stored: &str) -> Result<String, SecretBoxError> {
    let Some(encoded) = stored.strip_prefix(OBFUSCATED_PREFIX) else {
        return Err(SecretBoxError::InvalidFormat {
            message: format!("stored secret does not use {OBFUSCATED_PREFIX} format"),
        });
    };

    let payload = STANDARD
        .decode(encoded)
        .map_err(|error| SecretBoxError::InvalidFormat {
            message: format!("protected secret is not valid base64: {error}"),
        })?;

    if payload.len() <= NONCE_LEN {
        return Err(SecretBoxError::InvalidFormat {
            message: "protected secret payload is too short".into(),
        });
    }

    let (nonce, ciphertext) = payload.split_at(NONCE_LEN);
    let cipher = XChaCha20Poly1305::new(key.into());
    let plaintext = cipher
        .decrypt(XNonce::from_slice(nonce), ciphertext)
        .map_err(|error| SecretBoxError::Crypto {
            message: format!("decryption failed: {error}"),
        })?;

    String::from_utf8(plaintext).map_err(|error| SecretBoxError::InvalidFormat {
        message: format!("protected secret is not valid utf-8: {error}"),
    })
}
