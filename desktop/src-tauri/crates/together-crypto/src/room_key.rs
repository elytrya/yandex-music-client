use hkdf::Hkdf;
use sha2::Sha256;

use crate::encoding::{is_base32, to_base32};
use crate::SALT;

const TICKET_BYTES: usize = 10;

pub const TICKET_LEN: usize = 16;

const TICKET_INFO: &[u8] = b"room-ticket";
const KEY_INFO: &[u8] = b"room-key";

#[derive(Clone)]
pub struct RoomSecrets {

    pub ticket: String,

    pub key: [u8; 32],
}

pub fn normalize_invite(invite: &str) -> String {
    invite
        .split_whitespace()
        .map(|word| word.trim_matches('-').to_lowercase())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn derive(invite: &str) -> RoomSecrets {
    let normalized = normalize_invite(invite);
    let hkdf = Hkdf::<Sha256>::new(Some(SALT), normalized.as_bytes());

    let mut ticket = [0u8; TICKET_BYTES];
    hkdf.expand(TICKET_INFO, &mut ticket)
        .expect("длина билета всегда помещается в вывод HKDF-SHA256");

    let mut key = [0u8; 32];
    hkdf.expand(KEY_INFO, &mut key)
        .expect("32 байта всегда помещаются в вывод HKDF-SHA256");

    RoomSecrets {
        ticket: to_base32(&ticket),
        key,
    }
}

pub fn ticket(invite: &str) -> String {
    derive(invite).ticket
}

pub fn is_ticket(text: &str) -> bool {
    is_base32(text, TICKET_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite;

    #[test]
    fn same_invite_gives_same_secrets() {
        let invite = "olive canyon spirit velvet ladder onion";
        let first = derive(invite);
        let second = derive("  OLIVE   canyon Spirit velvet ladder onion ");
        assert_eq!(first.ticket, second.ticket);
        assert_eq!(first.key, second.key);
    }

    #[test]
    fn different_invites_give_different_secrets() {
        let first = derive(&invite::words());
        let second = derive(&invite::words());
        assert_ne!(first.ticket, second.ticket);
        assert_ne!(first.key, second.key);
    }

    #[test]
    fn ticket_has_expected_shape() {
        let secrets = derive(&invite::words());
        assert_eq!(secrets.ticket.len(), TICKET_LEN);
        assert!(is_ticket(&secrets.ticket));
        assert!(!is_ticket("olive canyon spirit velvet ladder onion"));
    }

    #[test]
    fn ticket_does_not_leak_key() {

        let secrets = derive("olive canyon spirit velvet ladder onion");
        let raw = crate::encoding::from_base32(&secrets.ticket).expect("билет читается");
        assert!(!secrets.key.windows(raw.len()).any(|part| part == raw));
    }
}
