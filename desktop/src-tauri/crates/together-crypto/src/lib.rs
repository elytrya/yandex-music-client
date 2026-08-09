#![forbid(unsafe_code)]

pub mod encoding;
pub mod identity;
pub mod invite;
pub mod payload;
pub mod room_key;

pub const SALT: &[u8] = b"mashiro-together/v1";

pub const PROTOCOL_VERSION: u8 = 1;
