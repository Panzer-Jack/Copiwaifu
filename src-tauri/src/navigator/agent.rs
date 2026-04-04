use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use tauri::AppHandle;

use super::{emit_all, state::NavigatorState, NavigatorStore};

pub const SESSION_TTL: Duration = Duration::from_secs(60);
pub const PERMISSION_TTL: Duration = Duration::from_secs(60);

pub fn session_key(agent: &super::events::AgentType, session_id: &str) -> String {
    format!("{}::{session_id}", agent.as_str())
}

pub fn start_cleanup_loop(app_handle: AppHandle, state: Arc<Mutex<NavigatorState>>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));

        let emissions = match state.lock() {
            Ok(mut navigator) => navigator.cleanup_stale(),
            Err(err) => {
                eprintln!("navigator cleanup lock poisoned: {err}");
                continue;
            }
        };

        emit_all(&app_handle, emissions);
    });
}

#[allow(dead_code)]
pub fn _assert_store_send_sync(_: &NavigatorStore) {}
