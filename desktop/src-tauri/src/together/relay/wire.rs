use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Serialize)]
#[serde(tag = "t")]
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth {
        #[serde(rename = "pub")]
        public: String,
        v: u8,
    },
    #[serde(rename = "proof")]
    Proof { sig: String },
    #[serde(rename = "room.create")]
    RoomCreate { ticket: String, nick: String },
    #[serde(rename = "room.join")]
    RoomJoin { invite: String, nick: String },
    #[serde(rename = "room.leave")]
    RoomLeave,
    #[serde(rename = "room.handoff")]
    RoomHandoff { to: u64 },
    #[serde(rename = "msg")]
    Msg { payload: Value },
    #[serde(rename = "ping")]
    Ping,
}

impl ClientMessage {

    pub fn to_frame(&self) -> String {
        serde_json::to_string(self).expect("сообщение провода всегда сериализуется")
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Peer {
    pub id: u64,
    #[serde(default)]
    pub nick: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "t")]
pub enum ServerMessage {
    #[serde(rename = "challenge")]
    Challenge { nonce: String },

    #[serde(rename = "session")]
    Session { ttl: u64 },
    #[serde(rename = "room")]
    Room {
        id: String,
        #[serde(default)]
        invite: Option<String>,
    },
    #[serde(rename = "welcome")]
    Welcome {
        id: u64,
        room: String,
        #[serde(default)]
        host: u64,
    },
    #[serde(rename = "peers")]
    Peers {
        peers: Vec<Peer>,
        #[serde(default)]
        host: u64,
    },
    #[serde(rename = "host")]
    Host { id: u64 },
    #[serde(rename = "msg")]
    Msg {
        from: u64,
        #[serde(default)]
        nick: String,
        payload: Value,
    },
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error {
        code: String,
        #[serde(default)]
        text: String,
    },

    #[serde(other)]
    Unknown,
}

pub fn describe(code: &str, text: &str) -> String {
    let head = match code {
        "rate_limit" => "слишком часто отправляем сообщения",
        "bad_sig" => "сервер не принял подпись",
        "challenge_expired" => "подпись ушла слишком поздно",
        "unauthorized" => "нужно пройти рукопожатие заново",
        "no_room" => "комнаты с таким кодом нет",
        "full" => "в комнате нет свободных мест",
        "not_host" => "передавать комнату может только ведущий",
        "no_peer" => "такого участника в комнате нет",
        "already_in_room" => "мы уже в комнате",
        "bad_ticket" => "код приглашения выглядит неправильно",
        "ip_limit" => "с этого адреса уже слишком много соединений",
        "too_large" => "сообщение слишком большое",
        "bad_json" | "bad_message" => "сервер не разобрал сообщение",
        _ => "сервер вернул ошибку",
    };

    if text.is_empty() {
        head.to_string()
    } else {
        format!("{head}: {text}")
    }
}

pub fn is_fatal(code: &str) -> bool {
    matches!(code, "too_large" | "ip_limit" | "unauthorized")
}
