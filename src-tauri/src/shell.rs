use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::{
    image::Image, menu::MenuBuilder, tray::TrayIconBuilder, App, AppHandle, Emitter, LogicalSize,
    Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
#[cfg(target_os = "macos")]
use tauri_plugin_autostart::ManagerExt as _;

use crate::navigator::NavigatorStore;

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const SETTINGS_WINDOW_LABEL: &str = "settings";
pub const SETTINGS_UPDATED_EVENT: &str = "settings:updated";
pub const WINDOW_VISIBILITY_CHANGED_EVENT: &str = "window:visibility-changed";
pub const DEFAULT_MODEL_URL: &str = "/Resources/Hiyori/Hiyori.model3.json";

const DEFAULT_MODEL_ENTRY_FILE: &str = "Hiyori.model3.json";
const SETTINGS_FILE_NAME: &str = "settings.json";
const NAME_MAX_LENGTH: usize = 16;
const TRAY_ID: &str = "copiwaifu-tray";
const CUSTOM_MODELS_DIR_NAME: &str = "custom-models";
const CURRENT_CUSTOM_MODEL_DIR_NAME: &str = "current";
const STAGED_CUSTOM_MODEL_DIR_NAME: &str = "current.staging";
const BACKUP_CUSTOM_MODEL_DIR_NAME: &str = "current.backup";
const MENU_OPEN_SETTINGS: &str = "open-settings";
const MENU_TOGGLE_VISIBILITY: &str = "toggle-visibility";
const MENU_EXIT: &str = "exit-app";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WindowSizePreset {
    Small,
    Medium,
    Large,
}

impl Default for WindowSizePreset {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguage {
    English,
    Chinese,
}

impl Default for AppLanguage {
    fn default() -> Self {
        Self::English
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MotionGroupOption {
    pub id: String,
    pub group: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub name: String,
    pub language: AppLanguage,
    pub auto_start: bool,
    pub model_directory: Option<String>,
    pub window_size: WindowSizePreset,
    pub action_group_bindings: BTreeMap<String, Option<String>>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "Yulia".to_string(),
            language: AppLanguage::English,
            auto_start: false,
            model_directory: None,
            window_size: WindowSizePreset::Medium,
            action_group_bindings: default_action_group_bindings(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelScanResult {
    pub model_entry_file: String,
    pub available_motion_groups: Vec<MotionGroupOption>,
    pub validation_passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedModelResult {
    pub imported_model_directory: String,
    pub model_scan: ModelScanResult,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    pub settings: AppSettings,
    pub model_scan: ModelScanResult,
    pub model_url: String,
    pub main_window_visible: bool,
    pub app_version: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WindowVisibilityPayload {
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct ShellState {
    settings: AppSettings,
    model_scan: ModelScanResult,
    main_window_visible: bool,
}

pub struct ShellStore(pub Arc<Mutex<ShellState>>);

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedAppSettings {
    name: Option<String>,
    language: Option<AppLanguage>,
    auto_start: Option<bool>,
    model_directory: Option<String>,
    window_size: Option<WindowSizePreset>,
    action_group_bindings: Option<BTreeMap<String, Option<String>>>,
    #[serde(rename = "actionBindings")]
    legacy_action_bindings: Option<BTreeMap<String, Option<String>>>,
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub fn get_app_bootstrap(
        app_handle: AppHandle,
        shell: State<'_, ShellStore>,
        navigator: State<'_, NavigatorStore>,
    ) -> Result<AppBootstrap, String> {
        let shell_state = shell.0.lock().map_err(|err| err.to_string())?;
        Ok(build_bootstrap(&app_handle, &shell_state, &navigator))
    }

    #[tauri::command]
    pub fn scan_model_directory(
        path: String,
        language: Option<AppLanguage>,
    ) -> Result<ModelScanResult, String> {
        scan_model_directory_path(Path::new(&path), None, language.unwrap_or_default())
    }

    #[tauri::command]
    pub fn import_model_directory(
        path: String,
        app_handle: AppHandle,
        language: Option<AppLanguage>,
    ) -> Result<ImportedModelResult, String> {
        import_model_directory_inner(&app_handle, Path::new(&path), language.unwrap_or_default())
    }

    #[tauri::command]
    pub fn scan_default_model(
        app_handle: AppHandle,
        language: Option<AppLanguage>,
    ) -> Result<ModelScanResult, String> {
        Ok(default_model_scan(
            &app_handle,
            None,
            language.unwrap_or_default(),
        ))
    }

    #[tauri::command]
    pub fn save_settings(
        settings: AppSettings,
        app_handle: AppHandle,
        shell: State<'_, ShellStore>,
        navigator: State<'_, NavigatorStore>,
    ) -> Result<AppBootstrap, String> {
        save_settings_inner(&app_handle, &shell, &navigator, settings)
    }

    #[tauri::command]
    pub async fn open_settings_window(
        app_handle: AppHandle,
        shell: State<'_, ShellStore>,
        navigator: State<'_, NavigatorStore>,
    ) -> Result<AppBootstrap, String> {
        open_or_focus_settings_window(&app_handle)?;
        let shell_state = shell.0.lock().map_err(|err| err.to_string())?;
        Ok(build_bootstrap(&app_handle, &shell_state, &navigator))
    }

    #[tauri::command]
    pub fn toggle_main_window_visibility(
        app_handle: AppHandle,
        shell: State<'_, ShellStore>,
        navigator: State<'_, NavigatorStore>,
    ) -> Result<AppBootstrap, String> {
        toggle_main_window_visibility_inner(&app_handle, &shell, &navigator)
    }

    #[tauri::command]
    pub fn exit_app(app_handle: AppHandle) -> Result<(), String> {
        app_handle.exit(0);
        Ok(())
    }
}

pub fn init(app: &mut App) -> tauri::Result<()> {
    let mut state = load_shell_state(app.handle())
        .map_err(|err| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, err)))?;
    let main_window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .or_else(|| app.webview_windows().into_values().next())
        .ok_or_else(|| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "main window not found",
            ))
        })?;

    apply_main_window_size(&main_window, &state.settings.window_size)?;
    state.main_window_visible = main_window.is_visible().unwrap_or(true);

    app.manage(ShellStore(Arc::new(Mutex::new(state))));
    create_tray(app.handle())?;

    Ok(())
}

pub fn current_model_directory(app_handle: &AppHandle) -> Option<PathBuf> {
    let shell = app_handle.try_state::<ShellStore>()?;
    let state = shell.0.lock().ok()?;
    state.settings.model_directory.as_deref().map(PathBuf::from)
}

fn import_model_directory_inner(
    app_handle: &AppHandle,
    source_directory: &Path,
    language: AppLanguage,
) -> Result<ImportedModelResult, String> {
    if !source_directory.exists() {
        return Err(model_directory_not_found_message(language));
    }
    if !source_directory.is_dir() {
        return Err(select_model_directory_message(language));
    }

    let custom_models_root = custom_models_root(app_handle)?;
    let current_directory = custom_models_root.join(CURRENT_CUSTOM_MODEL_DIR_NAME);
    let staging_directory = custom_models_root.join(STAGED_CUSTOM_MODEL_DIR_NAME);
    let backup_directory = custom_models_root.join(BACKUP_CUSTOM_MODEL_DIR_NAME);

    fs::create_dir_all(&custom_models_root).map_err(|err| err.to_string())?;
    remove_path_if_exists(&staging_directory).map_err(|err| err.to_string())?;
    remove_path_if_exists(&backup_directory).map_err(|err| err.to_string())?;

    if let Err(err) = copy_directory_contents(source_directory, &staging_directory) {
        let _ = remove_path_if_exists(&staging_directory);
        return Err(err.to_string());
    }

    let model_scan = match scan_model_directory_path(&staging_directory, None, language) {
        Ok(scan) => scan,
        Err(err) => {
            let _ = remove_path_if_exists(&staging_directory);
            return Err(err);
        }
    };

    if let Err(err) =
        replace_current_model_directory(&staging_directory, &current_directory, &backup_directory)
    {
        let _ = remove_path_if_exists(&staging_directory);
        let _ = remove_path_if_exists(&backup_directory);
        return Err(err.to_string());
    }

    remove_path_if_exists(&backup_directory).map_err(|err| err.to_string())?;

    Ok(ImportedModelResult {
        imported_model_directory: current_directory.to_string_lossy().to_string(),
        model_scan,
    })
}

fn save_settings_inner(
    app_handle: &AppHandle,
    shell: &State<'_, ShellStore>,
    navigator: &State<'_, NavigatorStore>,
    mut settings: AppSettings,
) -> Result<AppBootstrap, String> {
    normalize_user_settings(&mut settings)?;

    let model_scan = if let Some(model_directory) = settings.model_directory.as_deref() {
        scan_model_directory_path(Path::new(model_directory), None, settings.language)?
    } else {
        default_model_scan(app_handle, None, settings.language)
    };

    settings.action_group_bindings = sanitize_action_group_bindings(
        &settings.action_group_bindings,
        &model_scan.available_motion_groups,
    );
    persist_settings(app_handle, &settings)?;
    sync_autostart(app_handle, settings.auto_start)?;

    apply_main_window_size(&main_window(app_handle)?, &settings.window_size)
        .map_err(|err| err.to_string())?;

    {
        let mut shell_state = shell.0.lock().map_err(|err| err.to_string())?;
        shell_state.settings = settings;
        shell_state.model_scan = model_scan;
    }

    update_tray_menu(app_handle).map_err(|err| err.to_string())?;
    emit_settings_updated(app_handle, shell, navigator)?;

    let shell_state = shell.0.lock().map_err(|err| err.to_string())?;
    Ok(build_bootstrap(app_handle, &shell_state, navigator))
}

fn load_shell_state(app_handle: &AppHandle) -> Result<ShellState, String> {
    let mut settings = read_settings(app_handle)?;
    normalize_loaded_settings(&mut settings);

    let model_scan = match settings.model_directory.as_deref() {
        Some(model_directory) => {
            match scan_model_directory_path(Path::new(model_directory), None, settings.language) {
                Ok(scan) => scan,
                Err(err) => {
                    settings.model_directory = None;
                    settings.action_group_bindings = default_action_group_bindings();
                    default_model_scan(
                        app_handle,
                        Some(default_model_fallback_warning(settings.language, &err)),
                        settings.language,
                    )
                }
            }
        }
        None => default_model_scan(app_handle, None, settings.language),
    };

    settings.action_group_bindings = sanitize_action_group_bindings(
        &settings.action_group_bindings,
        &model_scan.available_motion_groups,
    );
    persist_settings(app_handle, &settings)?;
    sync_autostart(app_handle, settings.auto_start)?;

    Ok(ShellState {
        settings,
        model_scan,
        main_window_visible: true,
    })
}

fn read_settings(app_handle: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app_handle)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let persisted =
        serde_json::from_str::<PersistedAppSettings>(&raw).map_err(|err| err.to_string())?;
    Ok(merge_persisted_settings(persisted))
}

fn merge_persisted_settings(persisted: PersistedAppSettings) -> AppSettings {
    let mut settings = AppSettings::default();

    if let Some(name) = persisted.name {
        settings.name = name;
    }
    if let Some(language) = persisted.language {
        settings.language = language;
    }
    if let Some(auto_start) = persisted.auto_start {
        settings.auto_start = auto_start;
    }
    if persisted.model_directory.is_some() {
        settings.model_directory = persisted.model_directory;
    }
    if let Some(window_size) = persisted.window_size {
        settings.window_size = window_size;
    }
    if let Some(action_group_bindings) = persisted
        .action_group_bindings
        .or(persisted.legacy_action_bindings)
    {
        settings.action_group_bindings = merge_action_group_bindings(action_group_bindings);
    }

    settings
}

fn normalize_loaded_settings(settings: &mut AppSettings) {
    settings.name = settings.name.trim().to_string();
    if settings.name.is_empty() || settings.name.chars().count() > NAME_MAX_LENGTH {
        settings.name = AppSettings::default().name;
    }
    settings.action_group_bindings =
        merge_action_group_bindings(settings.action_group_bindings.clone());
}

fn normalize_user_settings(settings: &mut AppSettings) -> Result<(), String> {
    settings.name = settings.name.trim().to_string();
    if settings.name.is_empty() {
        return Err(name_required_message(settings.language));
    }
    if settings.name.chars().count() > NAME_MAX_LENGTH {
        return Err(name_too_long_message(settings.language, NAME_MAX_LENGTH));
    }
    settings.action_group_bindings =
        merge_action_group_bindings(settings.action_group_bindings.clone());
    Ok(())
}

fn persist_settings(app_handle: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app_handle)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let body = serde_json::to_string_pretty(settings).map_err(|err| err.to_string())?;
    fs::write(path, body).map_err(|err| err.to_string())
}

fn settings_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    app_handle
        .path()
        .app_config_dir()
        .map_err(|err| err.to_string())
        .map(|dir| dir.join(SETTINGS_FILE_NAME))
}

fn custom_models_root(app_handle: &AppHandle) -> Result<PathBuf, String> {
    app_handle
        .path()
        .app_data_dir()
        .map_err(|err| err.to_string())
        .map(|dir| dir.join(CUSTOM_MODELS_DIR_NAME))
}

fn create_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    let menu = tray_menu(app_handle, current_main_window_visibility(app_handle));
    let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("Copiwaifu")
        .icon(icon)
        .icon_as_template(false)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_OPEN_SETTINGS => {
                let _ = open_or_focus_settings_window(app);
            }
            MENU_TOGGLE_VISIBILITY => {
                if let (Some(shell), Some(navigator)) = (
                    app.try_state::<ShellStore>(),
                    app.try_state::<NavigatorStore>(),
                ) {
                    let _ = toggle_main_window_visibility_inner(app, &shell, &navigator);
                }
            }
            MENU_EXIT => app.exit(0),
            _ => {}
        })
        .build(app_handle)?;

    Ok(())
}

fn update_tray_menu(app_handle: &AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(tray_menu(
            app_handle,
            current_main_window_visibility(app_handle),
        )))?;
    }
    Ok(())
}

fn tray_menu(app_handle: &AppHandle, visible: bool) -> tauri::menu::Menu<tauri::Wry> {
    let language = current_language(app_handle);
    MenuBuilder::new(app_handle)
        .text(MENU_OPEN_SETTINGS, settings_menu_label(language))
        .text(
            MENU_TOGGLE_VISIBILITY,
            visibility_menu_label(visible, language),
        )
        .text(MENU_EXIT, exit_menu_label(language))
        .build()
        .expect("failed to build tray menu")
}

fn current_language(app_handle: &AppHandle) -> AppLanguage {
    app_handle
        .try_state::<ShellStore>()
        .and_then(|shell| shell.0.lock().ok().map(|state| state.settings.language))
        .unwrap_or_default()
}

fn open_or_focus_settings_window(app_handle: &AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.show().map_err(|err| err.to_string())?;
        window.set_focus().map_err(|err| err.to_string())?;
        window
            .set_title(settings_window_title(current_language(app_handle)))
            .map_err(|err| err.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app_handle,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title(settings_window_title(current_language(app_handle)))
    .inner_size(420.0, 620.0)
    .resizable(false)
    .focused(true)
    .build()
    .map_err(|err| err.to_string())?;

    Ok(())
}

fn toggle_main_window_visibility_inner(
    app_handle: &AppHandle,
    shell: &State<'_, ShellStore>,
    navigator: &State<'_, NavigatorStore>,
) -> Result<AppBootstrap, String> {
    let window = main_window(app_handle)?;
    let next_visible = !current_main_window_visibility(app_handle);

    if next_visible {
        window.show().map_err(|err| err.to_string())?;
        let _ = window.set_focus();
    } else {
        window.hide().map_err(|err| err.to_string())?;
    }

    {
        let mut shell_state = shell.0.lock().map_err(|err| err.to_string())?;
        shell_state.main_window_visible = next_visible;
    }

    update_tray_menu(app_handle).map_err(|err| err.to_string())?;
    app_handle
        .emit(
            WINDOW_VISIBILITY_CHANGED_EVENT,
            WindowVisibilityPayload {
                visible: next_visible,
            },
        )
        .map_err(|err| err.to_string())?;

    let shell_state = shell.0.lock().map_err(|err| err.to_string())?;
    Ok(build_bootstrap(app_handle, &shell_state, navigator))
}

fn emit_settings_updated(
    app_handle: &AppHandle,
    shell: &State<'_, ShellStore>,
    navigator: &State<'_, NavigatorStore>,
) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let language = shell
            .0
            .lock()
            .map_err(|err| err.to_string())?
            .settings
            .language;
        window
            .set_title(settings_window_title(language))
            .map_err(|err| err.to_string())?;
    }

    let shell_state = shell.0.lock().map_err(|err| err.to_string())?;
    let payload = build_bootstrap(app_handle, &shell_state, navigator);
    app_handle
        .emit(SETTINGS_UPDATED_EVENT, payload)
        .map_err(|err| err.to_string())
}

fn build_bootstrap(
    app_handle: &AppHandle,
    shell_state: &ShellState,
    navigator: &State<'_, NavigatorStore>,
) -> AppBootstrap {
    let server_port = navigator
        .0
        .lock()
        .ok()
        .and_then(|store| store.snapshot().server_port)
        .map(u16::from)
        .or_else(read_runtime_port);

    AppBootstrap {
        settings: shell_state.settings.clone(),
        model_scan: shell_state.model_scan.clone(),
        model_url: model_url_for(
            shell_state.settings.model_directory.is_some(),
            server_port,
            &shell_state.model_scan.model_entry_file,
        ),
        main_window_visible: shell_state.main_window_visible,
        app_version: app_handle.package_info().version.to_string(),
    }
}

fn model_url_for(has_custom_model: bool, port: Option<u16>, model_entry_file: &str) -> String {
    if has_custom_model {
        let port = port.unwrap_or(23333);
        return format!("http://127.0.0.1:{port}/model/current/{model_entry_file}");
    }
    DEFAULT_MODEL_URL.to_string()
}

fn main_window(app_handle: &AppHandle) -> Result<WebviewWindow, String> {
    app_handle
        .get_webview_window(MAIN_WINDOW_LABEL)
        .or_else(|| app_handle.webview_windows().into_values().next())
        .ok_or_else(|| "main window not found".to_string())
}

fn current_main_window_visibility(app_handle: &AppHandle) -> bool {
    main_window(app_handle)
        .ok()
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(true)
}

fn apply_main_window_size(window: &WebviewWindow, preset: &WindowSizePreset) -> tauri::Result<()> {
    let (width, height) = window_size_dimensions(preset);
    let size = LogicalSize::new(width, height);
    window.set_size(size)?;
    window.set_min_size(Some(size))?;
    window.set_max_size(Some(size))?;
    Ok(())
}

fn window_size_dimensions(preset: &WindowSizePreset) -> (f64, f64) {
    match preset {
        WindowSizePreset::Small => (160.0, 480.0),
        WindowSizePreset::Medium => (200.0, 600.0),
        WindowSizePreset::Large => (240.0, 720.0),
    }
}

fn default_action_group_bindings() -> BTreeMap<String, Option<String>> {
    agent_states()
        .into_iter()
        .map(|state| (state.to_string(), None))
        .collect()
}

fn merge_action_group_bindings(
    bindings: BTreeMap<String, Option<String>>,
) -> BTreeMap<String, Option<String>> {
    let mut merged = default_action_group_bindings();
    for (state, binding) in bindings {
        if merged.contains_key(&state) {
            merged.insert(state, binding);
        }
    }
    merged
}

fn sanitize_action_group_bindings(
    bindings: &BTreeMap<String, Option<String>>,
    _motion_groups: &[MotionGroupOption],
) -> BTreeMap<String, Option<String>> {
    merge_action_group_bindings(bindings.clone())
        .into_iter()
        .map(|(state, binding)| {
            let next_binding = binding
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            (state, next_binding)
        })
        .collect()
}

fn agent_states() -> [&'static str; 6] {
    [
        "idle",
        "thinking",
        "tool_use",
        "error",
        "complete",
        "needs_attention",
    ]
}

fn default_model_scan(
    app_handle: &AppHandle,
    warning: Option<String>,
    language: AppLanguage,
) -> ModelScanResult {
    if let Some(mut scan) = try_scan_default_model(app_handle, language) {
        if warning.is_some() {
            scan.validation_message = warning;
        }
        return scan;
    }

    ModelScanResult {
        model_entry_file: DEFAULT_MODEL_ENTRY_FILE.to_string(),
        available_motion_groups: Vec::new(),
        validation_passed: true,
        validation_message: warning,
    }
}

fn try_scan_default_model(
    app_handle: &AppHandle,
    language: AppLanguage,
) -> Option<ModelScanResult> {
    let candidates = default_model_directory_candidates(app_handle);

    for candidate in candidates {
        if candidate.exists() {
            if let Ok(scan) = scan_model_directory_path(&candidate, None, language) {
                return Some(scan);
            }
        }
    }

    None
}

fn default_model_directory_candidates(app_handle: &AppHandle) -> Vec<PathBuf> {
    let mut candidates = vec![
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../public/Resources/Hiyori"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist/Resources/Hiyori"),
    ];

    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        candidates.extend([
            resource_dir.join("Resources/Hiyori"),
            resource_dir.join("assets/Resources/Hiyori"),
            resource_dir.join("dist/Resources/Hiyori"),
            resource_dir.join("../Resources/Hiyori"),
        ]);
    }

    if let Ok(executable_path) = std::env::current_exe() {
        if let Some(executable_dir) = executable_path.parent() {
            candidates.extend([
                executable_dir.join("../Resources/Hiyori"),
                executable_dir.join("../resources/Resources/Hiyori"),
                executable_dir.join("Resources/Hiyori"),
            ]);
        }
    }

    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

pub fn scan_model_directory_path(
    directory: &Path,
    validation_message: Option<String>,
    language: AppLanguage,
) -> Result<ModelScanResult, String> {
    if !directory.exists() {
        return Err(model_directory_not_found_message(language));
    }
    if !directory.is_dir() {
        return Err(select_model_directory_message(language));
    }

    let mut entries = fs::read_dir(directory)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with(".model3.json"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    entries.sort();

    if entries.is_empty() {
        return Err(missing_model_entry_message(language));
    }
    if entries.len() > 1 {
        return Err(multiple_model_entries_message(language));
    }

    let entry_file = entries.remove(0);
    let entry_name = entry_file
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_model_entry_name_message(language))?
        .to_string();

    let raw = fs::read_to_string(&entry_file).map_err(|err| err.to_string())?;
    let json = serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| err.to_string())?;
    let file_refs = json
        .get("FileReferences")
        .and_then(|value| value.as_object())
        .ok_or_else(|| missing_file_references_message(language))?;

    let mut motion_groups = Vec::new();

    validate_declared_file(directory, file_refs.get("Moc"), language)?;
    validate_declared_file(directory, file_refs.get("Physics"), language)?;
    validate_declared_file(directory, file_refs.get("Pose"), language)?;
    validate_declared_file(directory, file_refs.get("UserData"), language)?;
    validate_texture_files(directory, file_refs.get("Textures"), language)?;
    validate_expression_files(directory, file_refs.get("Expressions"), language)?;
    validate_motion_files(
        directory,
        file_refs.get("Motions"),
        &mut motion_groups,
        language,
    )?;

    Ok(ModelScanResult {
        model_entry_file: entry_name,
        available_motion_groups: motion_groups,
        validation_passed: true,
        validation_message,
    })
}

fn validate_declared_file(
    directory: &Path,
    value: Option<&serde_json::Value>,
    language: AppLanguage,
) -> Result<(), String> {
    let Some(path) = value.and_then(|value| value.as_str()) else {
        return Ok(());
    };
    ensure_model_resource_exists(directory, path, language)
}

fn validate_texture_files(
    directory: &Path,
    value: Option<&serde_json::Value>,
    language: AppLanguage,
) -> Result<(), String> {
    let Some(files) = value.and_then(|value| value.as_array()) else {
        return Ok(());
    };

    for file in files {
        let path = file
            .as_str()
            .ok_or_else(|| invalid_textures_config_message(language))?;
        ensure_model_resource_exists(directory, path, language)?;
    }

    Ok(())
}

fn validate_expression_files(
    directory: &Path,
    value: Option<&serde_json::Value>,
    language: AppLanguage,
) -> Result<(), String> {
    let Some(expressions) = value.and_then(|value| value.as_array()) else {
        return Ok(());
    };

    for expression in expressions {
        if let Some(path) = expression.get("File").and_then(|value| value.as_str()) {
            ensure_model_resource_exists(directory, path, language)?;
        }
    }

    Ok(())
}

fn validate_motion_files(
    directory: &Path,
    value: Option<&serde_json::Value>,
    motion_groups: &mut Vec<MotionGroupOption>,
    language: AppLanguage,
) -> Result<(), String> {
    let Some(groups) = value.and_then(|value| value.as_object()) else {
        return Ok(());
    };
    let mut seen_groups = BTreeSet::new();

    for (group_name, items) in groups {
        let array = items
            .as_array()
            .ok_or_else(|| invalid_motion_group_message(language, group_name))?;

        for item in array {
            if let Some(path) = item.get("File").and_then(|value| value.as_str()) {
                ensure_model_resource_exists(directory, path, language)?;
            }
        }

        if seen_groups.insert(group_name.to_string()) {
            motion_groups.push(MotionGroupOption {
                id: group_name.to_string(),
                group: group_name.to_string(),
                label: group_name.to_string(),
            });
        }
    }

    Ok(())
}

fn ensure_model_resource_exists(
    directory: &Path,
    relative_path: &str,
    language: AppLanguage,
) -> Result<(), String> {
    let path = join_safe(directory, relative_path, language)?;
    if path.exists() {
        return Ok(());
    }
    Err(model_resource_missing_message(language, relative_path))
}

pub fn resolve_model_resource_path(
    directory: &Path,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let path = join_safe(directory, relative_path, AppLanguage::English)?;
    if path.is_file() {
        return Ok(path);
    }
    Err(model_resource_not_found_message(AppLanguage::English))
}

fn join_safe(base: &Path, relative_path: &str, language: AppLanguage) -> Result<PathBuf, String> {
    let mut path = base.to_path_buf();
    for component in Path::new(relative_path).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => path.push(segment),
            _ => return Err(invalid_model_path_message(language)),
        }
    }
    Ok(path)
}

fn copy_directory_contents(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let from_path = entry.path();
        let to_path = destination.join(entry.file_name());

        if entry_type.is_dir() {
            copy_directory_contents(&from_path, &to_path)?;
        } else if entry_type.is_file() {
            fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}

fn remove_path_if_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn replace_current_model_directory(
    staging_directory: &Path,
    current_directory: &Path,
    backup_directory: &Path,
) -> std::io::Result<()> {
    let had_current_directory = current_directory.exists();
    if had_current_directory {
        fs::rename(current_directory, backup_directory)?;
    }

    match fs::rename(staging_directory, current_directory) {
        Ok(()) => {
            if had_current_directory {
                remove_path_if_exists(backup_directory)?;
            }
            Ok(())
        }
        Err(err) => {
            if had_current_directory && backup_directory.exists() {
                let _ = fs::rename(backup_directory, current_directory);
            }
            Err(err)
        }
    }
}

fn settings_window_title(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Copiwaifu Settings",
        AppLanguage::Chinese => "Copiwaifu 设置",
    }
}

fn settings_menu_label(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Settings",
        AppLanguage::Chinese => "设置",
    }
}

fn exit_menu_label(language: AppLanguage) -> &'static str {
    match language {
        AppLanguage::English => "Exit",
        AppLanguage::Chinese => "退出",
    }
}

fn visibility_menu_label(visible: bool, language: AppLanguage) -> &'static str {
    match (visible, language) {
        (true, AppLanguage::English) => "Hide",
        (false, AppLanguage::English) => "Show",
        (true, AppLanguage::Chinese) => "隐藏",
        (false, AppLanguage::Chinese) => "显示",
    }
}

fn name_required_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "Name cannot be empty.".to_string(),
        AppLanguage::Chinese => "名字不能为空。".to_string(),
    }
}

fn name_too_long_message(language: AppLanguage, max_length: usize) -> String {
    match language {
        AppLanguage::English => format!("Name can be up to {max_length} characters."),
        AppLanguage::Chinese => format!("名字最多支持 {max_length} 个字符。"),
    }
}

fn default_model_fallback_warning(language: AppLanguage, error: &str) -> String {
    match language {
        AppLanguage::English => format!("Reverted to the built-in default model: {error}"),
        AppLanguage::Chinese => format!("已回退默认模型：{error}"),
    }
}

fn model_directory_not_found_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "Model directory does not exist.".to_string(),
        AppLanguage::Chinese => "模型目录不存在".to_string(),
    }
}

fn select_model_directory_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "Please choose a model directory.".to_string(),
        AppLanguage::Chinese => "请选择模型目录".to_string(),
    }
}

fn missing_model_entry_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "The folder root is missing a *.model3.json file.".to_string(),
        AppLanguage::Chinese => "目录顶层缺少 *.model3.json".to_string(),
    }
}

fn multiple_model_entries_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => {
            "The folder root contains multiple *.model3.json files.".to_string()
        }
        AppLanguage::Chinese => "目录顶层存在多个 *.model3.json".to_string(),
    }
}

fn invalid_model_entry_name_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "Invalid model entry filename.".to_string(),
        AppLanguage::Chinese => "模型入口文件名无效".to_string(),
    }
}

fn missing_file_references_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "model3.json is missing FileReferences.".to_string(),
        AppLanguage::Chinese => "model3.json 缺少 FileReferences".to_string(),
    }
}

fn invalid_textures_config_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "The Textures config is invalid.".to_string(),
        AppLanguage::Chinese => "Textures 配置格式无效".to_string(),
    }
}

fn invalid_motion_group_message(language: AppLanguage, group_name: &str) -> String {
    match language {
        AppLanguage::English => format!("Motion group {group_name} has an invalid config."),
        AppLanguage::Chinese => format!("动作组 {group_name} 配置格式无效"),
    }
}

fn model_resource_missing_message(language: AppLanguage, relative_path: &str) -> String {
    match language {
        AppLanguage::English => format!("Missing model resource: {relative_path}"),
        AppLanguage::Chinese => format!("模型资源缺失：{relative_path}"),
    }
}

fn model_resource_not_found_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "Model resource does not exist.".to_string(),
        AppLanguage::Chinese => "模型资源不存在".to_string(),
    }
}

fn invalid_model_path_message(language: AppLanguage) -> String {
    match language {
        AppLanguage::English => "Illegal model path.".to_string(),
        AppLanguage::Chinese => "模型路径非法".to_string(),
    }
}

fn sync_autostart(app_handle: &AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let manager = app_handle.autolaunch();
        if enabled {
            manager.enable().map_err(|err| err.to_string())?;
        } else {
            manager.disable().map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

fn read_runtime_port() -> Option<u16> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let candidates = [
        home.join(".copiwaifu/port"),
        PathBuf::from("/tmp/copiwaifu-port"),
    ];

    for path in candidates {
        if let Ok(value) = fs::read_to_string(path) {
            if let Ok(port) = value.trim().parse::<u16>() {
                return Some(port);
            }
        }
    }

    None
}
