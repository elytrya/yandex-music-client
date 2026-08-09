use std::fmt;

use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha256;

use crate::SALT;

pub const AUTH_CONTEXT: &[u8] = b"mashiro-together/auth/v1";

const IDENTITY_INFO: &[u8] = b"identity-ed25519";

pub const NONCE_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {

    Phrase,

    WordCount,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phrase => formatter.write_str("сид-фраза не разобралась"),
            Self::WordCount => formatter.write_str("нужно 12 или 24 слова"),
        }
    }
}

impl std::error::Error for IdentityError {}

pub fn normalize_phrase(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn keypair_from_phrase(phrase: &str) -> Result<SigningKey, IdentityError> {
    let normalized = normalize_phrase(phrase);
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, &normalized)
        .map_err(|_| IdentityError::Phrase)?;

    let words = mnemonic.word_count();
    if words != 12 && words != 24 {
        return Err(IdentityError::WordCount);
    }

    Ok(keypair_from_seed(&mnemonic.to_seed("")))
}

pub fn keypair_from_seed(seed: &[u8]) -> SigningKey {
    let hkdf = Hkdf::<Sha256>::new(Some(SALT), seed);
    let mut secret = [0u8; 32];
    hkdf.expand(IDENTITY_INFO, &mut secret)
        .expect("32 байта всегда помещаются в вывод HKDF-SHA256");
    SigningKey::from_bytes(&secret)
}

pub fn generate_phrase(words: usize) -> Result<String, IdentityError> {
    if words != 12 && words != 24 {
        return Err(IdentityError::WordCount);
    }

    let mnemonic = Mnemonic::generate(words).map_err(|_| IdentityError::Phrase)?;
    Ok(mnemonic.to_string())
}

pub fn auth_message(nonce: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(AUTH_CONTEXT.len() + nonce.len());
    message.extend_from_slice(AUTH_CONTEXT);
    message.extend_from_slice(nonce);
    message
}

pub fn sign_nonce(key: &SigningKey, nonce: &[u8]) -> Signature {
    key.sign(&auth_message(nonce))
}

pub fn verify_nonce(public: &VerifyingKey, nonce: &[u8], signature: &Signature) -> bool {
    public.verify_strict(&auth_message(nonce), signature).is_ok()
}

pub fn public_from_bytes(bytes: &[u8; 32]) -> Option<VerifyingKey> {
    VerifyingKey::from_bytes(bytes).ok()
}

pub fn signature_from_bytes(bytes: &[u8; 64]) -> Signature {
    Signature::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::to_base64;

    const PHRASE: &str =
        "legal winner thank year wave sausage worth useful legal winner thank yellow";

    #[test]
    fn phrase_gives_stable_key() {
        let first = keypair_from_phrase(PHRASE).expect("фраза верная");
        let second = keypair_from_phrase(&PHRASE.to_uppercase()).expect("регистр не важен");
        assert_eq!(
            first.verifying_key().to_bytes(),
            second.verifying_key().to_bytes()
        );
    }

    #[test]
    fn different_phrases_give_different_keys() {
        let first = keypair_from_phrase(PHRASE).expect("фраза верная");
        let other = generate_phrase(24).expect("генерация работает");
        let second = keypair_from_phrase(&other).expect("фраза верная");
        assert_ne!(
            first.verifying_key().to_bytes(),
            second.verifying_key().to_bytes()
        );
    }

    #[test]
    fn broken_phrase_is_rejected() {
        assert_eq!(
            keypair_from_phrase("это точно не сид фраза"),
            Err(IdentityError::Phrase)
        );
        assert_eq!(generate_phrase(15), Err(IdentityError::WordCount));
    }

    #[test]
    fn signature_round_trip() {
        let key = keypair_from_phrase(PHRASE).expect("фраза верная");
        let nonce = [42u8; NONCE_BYTES];
        let signature = sign_nonce(&key, &nonce);
        assert!(verify_nonce(&key.verifying_key(), &nonce, &signature));
    }

    #[test]
    fn foreign_nonce_does_not_pass() {
        let key = keypair_from_phrase(PHRASE).expect("фраза верная");
        let signature = sign_nonce(&key, &[1u8; NONCE_BYTES]);
        assert!(!verify_nonce(&key.verifying_key(), &[2u8; NONCE_BYTES], &signature));
    }

    #[test]
    fn public_key_fits_base64() {
        let key = keypair_from_phrase(PHRASE).expect("фраза верная");
        let text = to_base64(&key.verifying_key().to_bytes());
        assert_eq!(text.len(), 44);
    }
}
