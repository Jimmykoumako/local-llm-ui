use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tokio_util::sync::CancellationToken;

fn sessions() -> &'static Mutex<HashMap<String, CancellationToken>> {
    static SESSIONS: OnceLock<Mutex<HashMap<String, CancellationToken>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register(session_id: &str) -> CancellationToken {
    let token = CancellationToken::new();
    if let Ok(mut map) = sessions().lock() {
        map.insert(session_id.to_string(), token.clone());
    }
    token
}

pub fn cancel(session_id: &str) -> bool {
    if let Ok(map) = sessions().lock() {
        if let Some(token) = map.get(session_id) {
            token.cancel();
            return true;
        }
    }
    false
}

pub fn remove(session_id: &str) {
    if let Ok(mut map) = sessions().lock() {
        map.remove(session_id);
    }
}
