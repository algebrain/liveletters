use std::path::PathBuf;

use liveletters_secret_box::{SecretBox, SecretBoxError};
use tempfile::TempDir;

fn key_path(dir: &TempDir) -> PathBuf {
    dir.path().join("k.bin")
}

fn open_or_create_box(dir: &TempDir) -> SecretBox {
    SecretBox::open_or_create(&key_path(dir)).unwrap()
}

#[test]
fn obfuscation_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let box_ = open_or_create_box(&tmp);
    let token = box_.obfuscate("hunter2").unwrap();

    assert_ne!(token, "hunter2");
    assert!(SecretBox::is_obfuscated(&token));
    assert_eq!(box_.deobfuscate(&token).unwrap(), "hunter2");
}

#[test]
fn obfuscated_tokens_are_not_equal_for_same_plaintext() {
    let tmp = tempfile::tempdir().unwrap();
    let box_ = open_or_create_box(&tmp);

    let a = box_.obfuscate("same").unwrap();
    let b = box_.obfuscate("same").unwrap();
    assert_ne!(
        a, b,
        "XChaCha-Poly1305 с уникальным nonce даёт разные токены"
    );
    assert_eq!(box_.deobfuscate(&a).unwrap(), "same");
    assert_eq!(box_.deobfuscate(&b).unwrap(), "same");
}

#[test]
fn wrong_key_cannot_deobfuscate() {
    let tmp1 = tempfile::tempdir().unwrap();
    let tmp2 = tempfile::tempdir().unwrap();

    let box1 = open_or_create_box(&tmp1);
    let token = box1.obfuscate("secret").unwrap();

    let box2 = open_or_create_box(&tmp2);
    let err = box2.deobfuscate(&token).unwrap_err();
    assert!(matches!(err, SecretBoxError::Crypto { .. }));
}

#[test]
fn is_obfuscated_recognises_prefix() {
    assert!(SecretBox::is_obfuscated("obf:v1:abc"));
    assert!(!SecretBox::is_obfuscated("plain"));
    assert!(!SecretBox::is_obfuscated(""));
}

#[test]
fn deobfuscate_rejects_non_obfuscated_input() {
    let tmp = tempfile::tempdir().unwrap();
    let box_ = open_or_create_box(&tmp);
    let err = box_.deobfuscate("not-a-token").unwrap_err();
    assert!(matches!(err, SecretBoxError::InvalidFormat { .. }));
}

#[test]
fn deobfuscate_rejects_malformed_base64() {
    let tmp = tempfile::tempdir().unwrap();
    let box_ = open_or_create_box(&tmp);
    let err = box_.deobfuscate("obf:v1:###not-base64###").unwrap_err();
    assert!(matches!(err, SecretBoxError::InvalidFormat { .. }));
}

#[test]
fn deobfuscate_rejects_truncated_payload() {
    let tmp = tempfile::tempdir().unwrap();
    let box_ = open_or_create_box(&tmp);
    let err = box_.deobfuscate("obf:v1:AAA").unwrap_err();
    assert!(matches!(err, SecretBoxError::InvalidFormat { .. }));
}

#[test]
fn open_returns_io_error_for_missing_key() {
    let tmp = tempfile::tempdir().unwrap();
    let path = key_path(&tmp);
    let err = SecretBox::open(&path).unwrap_err();
    assert!(matches!(err, SecretBoxError::Io { .. }));
}

#[test]
fn open_rejects_key_with_wrong_length() {
    let tmp = tempfile::tempdir().unwrap();
    let path = key_path(&tmp);
    std::fs::write(&path, [0_u8; 16]).unwrap();
    let err = SecretBox::open(&path).unwrap_err();
    assert!(matches!(err, SecretBoxError::InvalidKeyLength { .. }));
}
