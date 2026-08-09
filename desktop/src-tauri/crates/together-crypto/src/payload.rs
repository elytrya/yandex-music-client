use std::fmt;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::encoding::{from_base64, to_base64};

pub const NONCE_BYTES: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadError {

    Format,

    Seal,

    Open,
}

impl fmt::Display for PayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format => formatter.write_str("конверт сообщения испорчен"),
            Self::Seal => formatter.write_str("не удалось зашифровать сообщение"),
            Self::Open => formatter.write_str("не удалось расшифровать сообщение"),
        }
    }
}

impl std::error::Error for PayloadError {}

pub fn seal(key: &[u8; 32], plain: &[u8]) -> Result<String, PayloadError> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));

    let mut nonce = [0u8; NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce);

    let cipher_text = cipher
        .encrypt(XNonce::from_slice(&nonce), plain)
        .map_err(|_| PayloadError::Seal)?;

    let mut envelope = Vec::with_capacity(NONCE_BYTES + cipher_text.len());
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&cipher_text);

    Ok(to_base64(&envelope))
}

pub fn open(key: &[u8; 32], envelope: &str) -> Result<Vec<u8>, PayloadError> {
    let raw = from_base64(envelope).ok_or(PayloadError::Format)?;
    if raw.len() <= NONCE_BYTES {
        return Err(PayloadError::Format);
    }

    let (nonce, body) = raw.split_at(NONCE_BYTES);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));

    cipher
        .decrypt(XNonce::from_slice(nonce), body)
        .map_err(|_| PayloadError::Open)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room_key;

    const PLAIN: &[u8] = br#"{"kind":"state","trackId":"123","positionMs":42000}"#;

    #[test]
    fn seal_and_open() {
        let secrets = room_key::derive("olive canyon spirit velvet ladder onion");
        let envelope = seal(&secrets.key, PLAIN).expect("шифруется");
        let opened = open(&secrets.key, &envelope).expect("расшифровывается");
        assert_eq!(opened, PLAIN);
    }

    #[test]
    fn nonce_is_fresh_every_time() {
        let secrets = room_key::derive("olive canyon spirit velvet ladder onion");
        let first = seal(&secrets.key, PLAIN).expect("шифруется");
        let second = seal(&secrets.key, PLAIN).expect("шифруется");
        assert_ne!(first, second);
    }

    #[test]
    fn foreign_key_does_not_open() {
        let mine = room_key::derive("olive canyon spirit velvet ladder onion");
        let other = room_key::derive("olive canyon spirit velvet ladder olive");
        let envelope = seal(&mine.key, PLAIN).expect("шифруется");
        assert_eq!(open(&other.key, &envelope), Err(PayloadError::Open));
    }

    #[test]
    fn spoiled_envelope_is_rejected() {
        let secrets = room_key::derive("olive canyon spirit velvet ladder onion");
        let envelope = seal(&secrets.key, PLAIN).expect("шифруется");

        let mut raw = crate::encoding::from_base64(&envelope).expect("base64 читается");
        let last = raw.len() - 1;
        raw[last] ^= 0x01;

        let spoiled = to_base64(&raw);
        assert_eq!(open(&secrets.key, &spoiled), Err(PayloadError::Open));
        assert_eq!(open(&secrets.key, "не base64"), Err(PayloadError::Format));
    }
}
