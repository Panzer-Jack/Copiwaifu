use tauri::{AppHandle, State};

use super::{emit_all, events::NavigatorStatus, hook_installer, NavigatorStore};

#[tauri::command]
pub fn respond_permission(
    permission_id: String,
    approved: bool,
    state: State<'_, NavigatorStore>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let emissions = state
        .0
        .lock()
        .map_err(|err| err.to_string())?
        .respond_permission(&permission_id, approved);

    emit_all(&app_handle, emissions);
    Ok(())
}

#[tauri::command]
pub fn get_agent_status(state: State<'_, NavigatorStore>) -> Result<NavigatorStatus, String> {
    let snapshot = state.0.lock().map_err(|err| err.to_string())?.snapshot();
    Ok(snapshot)
}

#[tauri::command]
pub fn uninstall_hooks() -> Result<(), String> {
    hook_installer::uninstall_hooks()
}
