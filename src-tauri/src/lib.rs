use tauri::{ActivationPolicy, Manager};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use tauri_nspanel::{cocoa::appkit::NSWindowCollectionBehavior, WebviewWindowExt};

mod navigator;

#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn elevate_desktop_pet_window(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let panel = window.to_panel().unwrap();

    panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel);

    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
    );

    panel.set_level(1000); // NSScreenSaverWindowLevel
    panel.order_front_regardless();

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .setup(|app| {
            navigator::init(app);

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(ActivationPolicy::Accessory);
                app.set_dock_visibility(false);
            }

            let window = app
                .get_webview_window("main")
                .or_else(|| app.webview_windows().into_values().next())
                .expect("failed to find the primary webview window");

            #[cfg(target_os = "macos")]
            elevate_desktop_pet_window(&window)?;

            #[cfg(not(target_os = "macos"))]
            window.set_always_on_top(true)?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            navigator::commands::respond_permission,
            navigator::commands::get_agent_status,
            navigator::commands::uninstall_hooks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
