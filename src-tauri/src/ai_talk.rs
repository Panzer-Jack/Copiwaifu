use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    navigator::{
        events::{AgentState, AgentType, AiTalkContext},
        NavigatorStore,
    },
    shell::{AiTalkSettings, AppLanguage, ShellStore, WindowSizePreset},
};

const AI_TALK_DEBUG_EVENT: &str = "ai-talk:debug";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTalkGenerateRequest {
    pub agent: AgentType,
    pub session_id: String,
    pub state: AgentState,
    pub window_size: WindowSizePreset,
    pub language: AppLanguage,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTalkGenerateResponse {
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SidecarRequest {
    config: SidecarModelConfig,
    context: AiTalkContext,
    language: AppLanguage,
    window_size: WindowSizePreset,
    max_length: usize,
    character_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SidecarModelConfig {
    provider: String,
    api_key: String,
    model_id: String,
    base_url: Option<String>,
    headers: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SidecarResponse {
    ok: bool,
    text: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiTalkDebugPayload {
    stage: String,
    data: serde_json::Value,
}

#[derive(Clone, Debug)]
struct SidecarRunOutput {
    text: Option<String>,
    stderr: String,
    response: Option<SidecarResponse>,
}

#[tauri::command]
pub async fn generate_ai_talk(
    request: AiTalkGenerateRequest,
    app_handle: AppHandle,
    shell: State<'_, ShellStore>,
    navigator: State<'_, NavigatorStore>,
) -> Result<Option<AiTalkGenerateResponse>, String> {
    let (config, character_name) = {
        let shell_state = shell.0.lock().map_err(|err| err.to_string())?;
        let Some(config) = build_model_config(&shell_state.settings.ai_talk) else {
            return Ok(None);
        };
        (config, shell_state.settings.name.clone())
    };

    let context = {
        let mut navigator = navigator.0.lock().map_err(|err| err.to_string())?;
        let Some(context) =
            navigator.claim_ai_talk_context(request.agent, &request.session_id, request.state)
        else {
            return Ok(None);
        };
        context
    };

    let Some(script_path) = resolve_sidecar_script(&app_handle) else {
        eprintln!("[ai_talk] sidecar script not found");
        return Ok(None);
    };

    let payload = SidecarRequest {
        config,
        context,
        language: request.language,
        window_size: request.window_size.clone(),
        max_length: max_ai_talk_length(&request.window_size, request.language),
        character_name,
    };

    eprintln!(
        "[ai_talk] sidecar request: {}",
        serde_json::to_string(&payload.redacted())
            .unwrap_or_else(|_| "<unserializable>".to_string())
    );
    emit_debug(&app_handle, "sidecar_request", payload.redacted());

    let output = tauri::async_runtime::spawn_blocking(move || run_sidecar(&script_path, &payload))
        .await
        .map_err(|err| err.to_string())?;
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            emit_debug(
                &app_handle,
                "sidecar_error",
                serde_json::json!({ "message": err }),
            );
            return Ok(None);
        }
    };

    emit_sidecar_debug_logs(&app_handle, &output.stderr);
    if let Some(response) = &output.response {
        emit_debug(
            &app_handle,
            "sidecar_response",
            serde_json::to_value(response).unwrap_or_else(|_| serde_json::json!({})),
        );
    }

    let normalized = output.text.and_then(|text| normalize_sidecar_text(&text));
    emit_debug(
        &app_handle,
        "generate_result",
        serde_json::json!({ "text": normalized.clone() }),
    );

    Ok(normalized.map(|text| AiTalkGenerateResponse { text }))
}

fn emit_debug(app_handle: &AppHandle, stage: &str, data: serde_json::Value) {
    let _ = app_handle.emit(
        AI_TALK_DEBUG_EVENT,
        AiTalkDebugPayload {
            stage: stage.to_string(),
            data,
        },
    );
}

fn emit_sidecar_debug_logs(app_handle: &AppHandle, stderr: &str) {
    for line in stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if let Some(data) = parse_prefixed_json(line, "[AI Talk sidecar] provider request ") {
            emit_debug(app_handle, "provider_request", data);
            continue;
        }

        if let Some(data) = parse_prefixed_json(line, "[AI Talk sidecar] provider response ") {
            emit_debug(app_handle, "provider_response", data);
            continue;
        }

        if let Some(data) = parse_prefixed_json(line, "[AI Talk sidecar] provider retry request ") {
            emit_debug(app_handle, "provider_retry_request", data);
            continue;
        }

        emit_debug(
            app_handle,
            "sidecar_stderr",
            serde_json::json!({ "text": line }),
        );
    }
}

fn parse_prefixed_json(line: &str, prefix: &str) -> Option<serde_json::Value> {
    let value = line.strip_prefix(prefix)?;
    serde_json::from_str(value).ok()
}

impl SidecarRequest {
    fn redacted(&self) -> serde_json::Value {
        serde_json::json!({
            "config": {
                "provider": &self.config.provider,
                "modelId": &self.config.model_id,
                "hasApiKey": !self.config.api_key.is_empty(),
                "baseUrl": &self.config.base_url,
                "headerKeys": self.config.headers.keys().collect::<Vec<_>>(),
            },
            "context": &self.context,
            "language": self.language,
            "windowSize": &self.window_size,
            "maxLength": self.max_length,
            "characterName": &self.character_name,
        })
    }
}

fn build_model_config(settings: &AiTalkSettings) -> Option<SidecarModelConfig> {
    if !settings.enabled || settings.api_key.is_empty() || settings.model_id.is_empty() {
        return None;
    }

    if settings.provider == "openai-compatible" && settings.base_url.is_none() {
        return None;
    }

    Some(SidecarModelConfig {
        provider: settings.provider.clone(),
        api_key: settings.api_key.clone(),
        model_id: settings.model_id.clone(),
        base_url: settings.base_url.clone(),
        headers: settings.headers.clone(),
    })
}

fn run_sidecar(script_path: &Path, payload: &SidecarRequest) -> Result<SidecarRunOutput, String> {
    let input = serde_json::to_vec(payload).map_err(|err| err.to_string())?;
    let mut child = Command::new("node")
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(&input).map_err(|err| err.to_string())?;
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !stderr.trim().is_empty() {
        eprintln!("{}", stderr.trim());
    }
    if !output.status.success() {
        return Ok(SidecarRunOutput {
            text: None,
            stderr,
            response: None,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response = serde_json::from_str::<SidecarResponse>(stdout.trim())
        .map_err(|err| format!("failed_to_parse_sidecar_response: {err}"))?;
    if !response.ok {
        return Ok(SidecarRunOutput {
            text: None,
            stderr,
            response: Some(response),
        });
    }

    eprintln!(
        "[ai_talk] sidecar response: {}",
        serde_json::to_string(&response).unwrap_or_else(|_| "<unserializable>".to_string())
    );

    let text = response.text.clone();
    Ok(SidecarRunOutput {
        text,
        stderr,
        response: Some(response),
    })
}

fn normalize_sidecar_text(text: &str) -> Option<String> {
    let text = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let text = text
        .trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == '`')
        .trim()
        .to_string();

    (!text.is_empty()).then_some(text)
}

fn resolve_sidecar_script(app_handle: &AppHandle) -> Option<PathBuf> {
    let dev_script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../sidecar/ai-runtime/src/main.mjs");
    if dev_script.exists() {
        return Some(dev_script);
    }

    let resource_dir = app_handle.path().resource_dir().ok()?;
    [
        resource_dir.join("sidecar/ai-runtime/bundle/main.mjs"),
        resource_dir.join("ai-runtime/bundle/main.mjs"),
        resource_dir.join("bundle/main.mjs"),
        resource_dir.join("main.mjs"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn max_ai_talk_length(window_size: &WindowSizePreset, language: AppLanguage) -> usize {
    let cjk = matches!(language, AppLanguage::Chinese | AppLanguage::Japanese);
    match (window_size, cjk) {
        (WindowSizePreset::Tiny, true) => 24,
        (WindowSizePreset::Tiny, false) => 45,
        (WindowSizePreset::Small, true) => 36,
        (WindowSizePreset::Small, false) => 70,
        (WindowSizePreset::Medium, true) => 42,
        (WindowSizePreset::Medium, false) => 80,
        (WindowSizePreset::Large, true) => 60,
        (WindowSizePreset::Large, false) => 110,
    }
}
