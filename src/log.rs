use std::sync::Mutex;

#[derive(Clone)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
}

static LOG: Mutex<Option<(String, LogLevel)>> = Mutex::new(None);

pub fn log(message: impl Into<String>, level: LogLevel) {
    let mut data = LOG.lock().unwrap();
    *data = Some((message.into(), level));
}

pub fn current() -> Option<(String, LogLevel)> {
    LOG.lock().ok()?.clone()
}

pub fn clear_log() {
    let mut data = LOG.lock().unwrap();
    *data = None;
}
