use serde_json::{json, Value};
use together_crypto::room_key::{self, RoomSecrets};
use together_crypto::{invite as invite_codes, payload};

pub fn secrets(invite: &str) -> Result<RoomSecrets, String> {
    if !invite_codes::looks_valid(invite) {
        return Err(String::from("код приглашения выглядит неправильно"));
    }

    Ok(room_key::derive(invite))
}

pub fn fresh_invite() -> String {
    invite_codes::words()
}

pub fn short_invite() -> String {
    invite_codes::short()
}

pub fn seal(key: &[u8; 32], value: &Value) -> Result<Value, String> {
    let plain = serde_json::to_vec(value).map_err(|_| String::from("сообщение не собралось"))?;
    let envelope = payload::seal(key, &plain).map_err(|error| error.to_string())?;
    Ok(json!({ "enc": envelope }))
}

pub fn open(key: &[u8; 32], value: &Value) -> Result<Value, String> {
    let envelope = value
        .get("enc")
        .and_then(Value::as_str)
        .ok_or_else(|| String::from("сообщение пришло без конверта"))?;

    let plain = payload::open(key, envelope).map_err(|error| error.to_string())?;
    serde_json::from_slice(&plain).map_err(|_| String::from("внутри конверта не JSON"))
}
