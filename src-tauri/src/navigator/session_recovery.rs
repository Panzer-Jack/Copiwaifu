use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::{
    events::{AgentEvent, AgentType, EventData, EventType},
    state::NavigatorState,
};

const SESSION_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub fn recover(state: &mut NavigatorState) {
    let sessions_dir = match home_sessions_dir() {
        Some(p) => p,
        None => return,
    };

    if !sessions_dir.exists() {
        return;
    }

    let entries = match fs::read_dir(&sessions_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("[session_recovery] read_dir failed: {err}");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            recover_session(state, &path);
        }
    }
}

fn recover_session(state: &mut NavigatorState, path: &PathBuf) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("[session_recovery] read failed {path:?}: {err}");
            return;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("[session_recovery] parse failed {path:?}: {err}");
            return;
        }
    };

    // 已结束的 session 直接删除
    if json.get("endedAt").is_some_and(|v| !v.is_null()) {
        let _ = fs::remove_file(path);
        return;
    }

    // 超过 24 小时的 session 删除
    if let Some(last_updated_ms) = json["lastUpdated"].as_i64() {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let age = Duration::from_millis((now_ms - last_updated_ms).max(0) as u64);
        if age > SESSION_MAX_AGE {
            let _ = fs::remove_file(path);
            return;
        }
    }

    let session_id = match json["sessionId"].as_str() {
        Some(s) => s.to_string(),
        None => return,
    };

    let agent = match json["agent"].as_str() {
        Some("claude-code") => AgentType::ClaudeCode,
        Some("copilot") => AgentType::Copilot,
        Some("codex") => AgentType::Codex,
        _ => return,
    };

    let event_type = match json["status"].as_str() {
        Some("working") => EventType::Thinking,
        Some("error") => EventType::Error,
        Some("completed") => EventType::Complete,
        _ => EventType::SessionStart,
    };

    let last_event = &json["lastEvent"];
    let tool_name = last_event["toolName"].as_str().map(str::to_string);
    let summary = last_event["summary"].as_str().map(str::to_string);

    let event = AgentEvent {
        agent,
        session_id,
        event: event_type,
        data: EventData {
            tool_name,
            summary,
            working_directory: json["workingDirectory"].as_str().map(str::to_string),
            session_title: json["sessionTitle"].as_str().map(str::to_string),
            needs_attention: json["needsAttention"].as_bool(),
        },
    };

    state.apply_event(event);
}

fn home_sessions_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".copiwaifu").join("sessions"))
}
