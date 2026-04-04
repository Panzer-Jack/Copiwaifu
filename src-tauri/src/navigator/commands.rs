use tauri::State;

use super::{events::NavigatorStatus, hook_installer, NavigatorStore};

#[tauri::command]
pub fn get_agent_status(state: State<'_, NavigatorStore>) -> Result<NavigatorStatus, String> {
    let snapshot = state.0.lock().map_err(|err| err.to_string())?.snapshot();
    Ok(snapshot)
}

#[tauri::command]
pub fn uninstall_hooks() -> Result<(), String> {
    hook_installer::uninstall_hooks()
}
