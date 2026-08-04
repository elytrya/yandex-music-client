use std::sync::Mutex;

#[derive(Clone)]
pub struct Session {
    pub token: String,
    pub uid: i64,
}

#[derive(Default)]
pub struct AppState {
    inner: Mutex<Option<Session>>,
}

impl AppState {
    pub fn session(&self) -> Result<Session, String> {
        self.inner
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "Не авторизован".to_string())
    }

    pub fn set_session(&self, session: Session) {
        *self.inner.lock().unwrap() = Some(session);
    }

    pub fn clear(&self) {
        *self.inner.lock().unwrap() = None;
    }
}
