use bip39::Language;
use rand::rngs::OsRng;
use rand::RngCore;

use crate::encoding::to_base32;

pub const INVITE_WORDS: usize = 6;

pub const INVITE_SHORT_LEN: usize = 10;

const SHORT_BYTES: usize = 6;

pub fn words() -> String {
    let list = Language::English.word_list();
    let mut raw = [0u8; INVITE_WORDS * 2];
    OsRng.fill_bytes(&mut raw);

    (0..INVITE_WORDS)
        .map(|index| {
            let pair = u16::from_be_bytes([raw[index * 2], raw[index * 2 + 1]]);
            list[usize::from(pair) % list.len()]
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn short() -> String {
    let mut raw = [0u8; SHORT_BYTES];
    OsRng.fill_bytes(&mut raw);
    to_base32(&raw)
}

pub fn looks_valid(invite: &str) -> bool {
    let normalized = crate::room_key::normalize_invite(invite);
    if normalized.is_empty() {
        return false;
    }

    let words = normalized.split(' ').count();
    words >= 2 || normalized.len() >= INVITE_SHORT_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_invite_has_six_words() {
        let invite = words();
        assert_eq!(invite.split(' ').count(), INVITE_WORDS);
        assert!(looks_valid(&invite));
    }

    #[test]
    fn word_invites_differ() {
        assert_ne!(words(), words());
    }

    #[test]
    fn short_invite_has_ten_symbols() {
        let invite = short();
        assert_eq!(invite.len(), INVITE_SHORT_LEN);
        assert!(looks_valid(&invite));
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(!looks_valid("   "));
        assert!(!looks_valid("abc"));
    }
}
