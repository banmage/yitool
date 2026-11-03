use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HmacError {
    #[error("HMAC 密钥不可用")]
    KeyUnavailable,
    #[error("HMAC 计算失败")]
    InvalidKey,
}

/// 本地密钥提供者。真实实现应在阶段 1 接入 OS Keychain；本阶段用 mock 验证契约。
pub trait KeyProvider {
    fn key_id(&self) -> String;
    fn key(&self) -> Result<[u8; 32], HmacError>;
}

#[derive(Debug, Clone)]
pub struct MockKeyProvider {
    pub key_id: String,
    key: [u8; 32],
}

impl MockKeyProvider {
    pub fn new(key_id: impl Into<String>, key: [u8; 32]) -> Self {
        Self {
            key_id: key_id.into(),
            key,
        }
    }
}

impl KeyProvider for MockKeyProvider {
    fn key_id(&self) -> String {
        self.key_id.clone()
    }

    fn key(&self) -> Result<[u8; 32], HmacError> {
        Ok(self.key)
    }
}

/// 问事文本 → HMAC-SHA256（NFC 规范化，十六进制小写）。
/// 原文不得写入日志；只输出 HMAC 与 key_id。
pub fn question_hmac(key: &[u8], text: &str) -> Result<String, HmacError> {
    let normalized: String = text.nfc().collect();
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| HmacError::InvalidKey)?;
    mac.update(b"QUESTION-HMAC-v1\0");
    mac.update(normalized.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfc_normalization_stable() {
        let composed = "é问事";
        let decomposed = "e\u{0301}问事";
        let key = [7u8; 32];
        assert_eq!(
            question_hmac(&key, composed).unwrap(),
            question_hmac(&key, decomposed).unwrap()
        );
    }

    #[test]
    fn different_text_different_hmac() {
        let key = [7u8; 32];
        assert_ne!(
            question_hmac(&key, "问财运").unwrap(),
            question_hmac(&key, "问学业").unwrap()
        );
    }
}
