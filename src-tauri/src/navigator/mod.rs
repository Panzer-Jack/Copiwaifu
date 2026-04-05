use std::sync::{Arc, Mutex};

use tauri::{App, AppHandle, Emitter, Manager};

pub mod agent;
pub mod commands;
pub mod events;
mod hook_helpers;
pub mod hook_installer;
pub mod server;
pub mod session_recovery;
pub mod state;

use events::NavigatorEmission;
use state::NavigatorState;

pub struct NavigatorStore(pub Arc<Mutex<NavigatorState>>);

pub fn init(app: &mut App) {
    let state = Arc::new(Mutex::new(NavigatorState::new()));

    // 启动时从 session 文件恢复状态
    if let Ok(mut navigator) = state.lock() {
        session_recovery::recover(&mut navigator);
    }

    app.manage(NavigatorStore(state.clone()));
    server::start(app.handle().clone(), state.clone());
    agent::start_cleanup_loop(app.handle().clone(), state);

    if let Err(err) = hook_installer::install_hooks() {
        eprintln!("navigator hook installation failed: {err}");
    }
}

pub fn emit_all(app_handle: &AppHandle, emissions: Vec<NavigatorEmission>) {
    for emission in emissions {
        match emission {
            NavigatorEmission::StateChange(payload) => {
                let _ = app_handle.emit("agent:state-change", payload);
            }
        }
    }
}
