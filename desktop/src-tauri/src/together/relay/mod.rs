pub mod client;
pub mod commands;
pub mod crypto;
pub mod keys;
pub mod wire;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

pub const DEFAULT_NICK: &str = "слушатель";

pub const NICK_LIMIT: usize = 32;

#[derive(Clone, Debug, Serialize)]
pub struct PeerInfo {
    pub id: u64,
    pub nick: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayStatus {

    pub connected: bool,

    pub address: String,

    pub room: String,

    pub invite: String,
    pub nick: String,

    pub self_id: u64,

    pub host: u64,
    pub peers: Vec<PeerInfo>,
}

impl RelayStatus {

    pub fn is_host(&self) -> bool {
        self.connected && self.self_id != 0 && self.self_id == self.host
    }
}

pub enum Outgoing {

    Payload(Box<Value>),

    Handoff(u64),

    Leave,
}

struct Handle {
    outgoing: UnboundedSender<Outgoing>,
}

#[derive(Default)]
struct Inner {
    handle: Mutex<Option<Handle>>,
    status: Mutex<RelayStatus>,

    epoch: AtomicU64,
}

#[derive(Clone, Default)]
pub struct RelayState {
    inner: Arc<Inner>,
}

impl RelayState {
    fn handle(&self) -> MutexGuard<'_, Option<Handle>> {
        self.inner
            .handle
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn snapshot(&self) -> MutexGuard<'_, RelayStatus> {
        self.inner
            .status
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn status(&self) -> RelayStatus {
        self.snapshot().clone()
    }

    pub fn update<F>(&self, edit: F) -> RelayStatus
    where
        F: FnOnce(&mut RelayStatus),
    {
        let mut status = self.snapshot();
        edit(&mut status);
        status.clone()
    }

    pub fn begin(&self, outgoing: UnboundedSender<Outgoing>, status: RelayStatus) -> u64 {
        let mut handle = self.handle();
        *handle = Some(Handle { outgoing });
        drop(handle);

        *self.snapshot() = status;
        self.inner.epoch.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn owns(&self, epoch: u64) -> bool {
        self.inner.epoch.load(Ordering::SeqCst) == epoch
    }

    pub fn finish(&self, epoch: u64) {
        if !self.owns(epoch) {
            return;
        }

        let mut handle = self.handle();
        handle.take();
        drop(handle);

        *self.snapshot() = RelayStatus::default();
    }

    pub fn is_active(&self) -> bool {
        self.handle().is_some()
    }

    pub fn send(&self, order: Outgoing) -> Result<(), String> {
        let guard = self.handle();
        let handle = guard
            .as_ref()
            .ok_or_else(|| String::from("соединения с сервером нет"))?;

        handle
            .outgoing
            .send(order)
            .map_err(|_| String::from("соединение уже закрыто"))
    }

    pub fn stop(&self) {
        let mut handle = self.handle();
        if let Some(live) = handle.take() {
            let _ = live.outgoing.send(Outgoing::Leave);
        }
        drop(handle);

        *self.snapshot() = RelayStatus::default();
    }
}
