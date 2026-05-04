use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::json;

use super::{
    events::{AgentType, SessionPhase},
    state::SessionSnapshot,
};

const SESSION_DIR_NAME: &str = "sessions";

pub fn persist_snapshot(session: &SessionSnapshot) -> Result<(), String> {
    let file = session_file_path(session.agent, &session.session_id)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let last_event = session.events.last().map(|event| {
        json!({
            "type": event.event_type,
            "timestamp": event.timestamp_ms,
            "toolName": event.tool_name,
            "summary": event.summary,
            "informative": event.informative,
        })
    });

    let body = json!({
        "sessionId": &session.session_id,
        "agent": session.agent,
        "status": status_for_phase(session.phase),
        "startedAt": session.started_at_ms,
        "lastUpdated": session.updated_at_ms,
        "workingDirectory": &session.working_directory,
        "sessionTitle": &session.session_title,
        "needsAttention": session.needs_attention.unwrap_or(false),
        "lastEvent": last_event,
        "events": &session.events,
        "lastMeaningfulSummary": &session.last_meaningful_summary,
        "aiTalkContext": &session.ai_talk_context,
    });

    atomic_write_json(&file, &body)
}

pub fn mark_session_ended(
    agent: AgentType,
    session_id: &str,
    ended_at_ms: u64,
) -> Result<(), String> {
    let file = session_file_path(agent, session_id)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut body = if file.exists() {
        fs::read_to_string(&file)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };

    body["sessionId"] = json!(session_id);
    body["agent"] = json!(agent);
    body["status"] = json!("idle");
    body["lastUpdated"] = json!(ended_at_ms);
    body["endedAt"] = json!(ended_at_ms);

    atomic_write_json(&file, &body)
}

fn status_for_phase(phase: SessionPhase) -> &'static str {
    match phase {
        SessionPhase::Idle => "idle",
        SessionPhase::Processing | SessionPhase::RunningTool | SessionPhase::WaitingAttention => {
            "working"
        }
        SessionPhase::Completed => "completed",
        SessionPhase::Error => "error",
    }
}

fn session_file_path(agent: AgentType, session_id: &str) -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".copiwaifu")
        .join(SESSION_DIR_NAME)
        .join(format!(
            "{}_{}.json",
            agent.as_str(),
            safe_session_id(session_id)
        )))
}

fn safe_session_id(session_id: &str) -> String {
    session_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn atomic_write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(&tmp, body).map_err(|err| err.to_string())?;
    fs::rename(tmp, path).map_err(|err| err.to_string())
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}
