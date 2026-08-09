use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7331;
pub const HOST_ID: u64 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Off,
    Host,
    Guest,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Off
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: u64,
    pub nick: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TogetherStatus {
    pub mode: Mode,
    pub port: u16,
    pub nick: String,
    pub address: Option<String>,
    pub self_id: u64,
    pub peers: Vec<PeerInfo>,
}

pub struct Peer {
    pub info: PeerInfo,
    pub tx: Sender<String>,
    pub stream: TcpStream,
}

#[derive(Default)]
pub struct Room {
    pub mode: Mode,
    pub nick: String,
    pub port: u16,
    pub address: Option<String>,
    pub self_id: u64,
    pub next_id: u64,
    pub peers: HashMap<u64, Peer>,
    pub roster: Vec<PeerInfo>,
    pub out: Option<Sender<String>>,
    pub stream: Option<TcpStream>,
    pub alive: Option<Arc<AtomicBool>>,
}

impl Room {
    pub fn snapshot(&self) -> TogetherStatus {
        TogetherStatus {
            mode: self.mode,
            port: if self.port == 0 {
                DEFAULT_PORT
            } else {
                self.port
            },
            nick: self.nick.clone(),
            address: self.address.clone(),
            self_id: self.self_id,
            peers: self.roster.clone(),
        }
    }
}

#[derive(Default)]
pub struct TogetherState {
    pub inner: Mutex<Room>,
}
