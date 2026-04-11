use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use tauri::AppHandle;

use super::{
    emit_all,
    events::{AgentEvent, AgentType, EventData, EventType},
    hook_helpers::home_dir,
    state::NavigatorState,
};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const RECENT_SESSION_MAX_AGE: Duration = Duration::from_secs(90);
const MAX_RECENT_CODEX_ROLLOUTS: usize = 8;
const MAX_RECENT_PROVIDER_SESSIONS: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedSession {
    agent: AgentType,
    session_id: String,
    event: EventType,
    tool_name: Option<String>,
    summary: Option<String>,
    working_directory: Option<String>,
    session_title: Option<String>,
}

impl ObservedSession {
    fn key(&self) -> String {
        format!("{}::{}", self.agent.as_str(), self.session_id)
    }

    fn into_agent_event(self) -> AgentEvent {
        AgentEvent {
            agent: self.agent,
            session_id: self.session_id,
            event: self.event,
            data: EventData {
                tool_name: self.tool_name,
                summary: self.summary,
                working_directory: self.working_directory,
                session_title: self.session_title,
                needs_attention: Some(false),
            },
        }
    }
}

#[derive(Default)]
struct CodexRolloutState {
    session_id: Option<String>,
    working_directory: Option<String>,
    session_title: Option<String>,
    current_turn_active: bool,
    last_completion_summary: Option<String>,
}

pub fn start(app_handle: AppHandle, state: Arc<Mutex<NavigatorState>>) {
    thread::spawn(move || {
        let mut previous = HashMap::<String, ObservedSession>::new();

        loop {
            thread::sleep(POLL_INTERVAL);

            let observed = scan_recent_sessions().unwrap_or_default();
            let mut changed: Vec<ObservedSession> = observed
                .iter()
                .filter_map(|(session_key, snapshot)| match previous.get(session_key) {
                    Some(last) if last == snapshot => None,
                    _ => Some(snapshot.clone()),
                })
                .collect();

            for (session_key, snapshot) in &previous {
                if observed.contains_key(session_key) {
                    continue;
                }
                if let Some(finalized) = finalize_disappeared(snapshot) {
                    changed.push(finalized);
                }
            }

            previous = observed;

            if changed.is_empty() {
                continue;
            }

            let emissions = match state.lock() {
                Ok(mut navigator) => changed
                    .into_iter()
                    .flat_map(|snapshot| navigator.apply_event(snapshot.into_agent_event()))
                    .collect(),
                Err(err) => {
                    eprintln!("navigator reconcile lock poisoned: {err}");
                    continue;
                }
            };

            emit_all(&app_handle, emissions);
        }
    });
}

fn finalize_disappeared(snapshot: &ObservedSession) -> Option<ObservedSession> {
    if snapshot.event == EventType::Complete {
        return None;
    }

    Some(ObservedSession {
        agent: snapshot.agent,
        session_id: snapshot.session_id.clone(),
        event: EventType::Complete,
        tool_name: snapshot.tool_name.clone(),
        summary: snapshot
            .summary
            .clone()
            .or_else(|| snapshot.session_title.clone()),
        working_directory: snapshot.working_directory.clone(),
        session_title: snapshot.session_title.clone(),
    })
}

fn scan_recent_sessions() -> Result<HashMap<String, ObservedSession>, String> {
    let mut sessions = HashMap::new();
    for snapshot in scan_recent_codex_sessions()? {
        sessions.insert(snapshot.key(), snapshot);
    }
    for snapshot in scan_recent_gemini_sessions()? {
        sessions.insert(snapshot.key(), snapshot);
    }
    for snapshot in scan_recent_opencode_sessions()? {
        sessions.insert(snapshot.key(), snapshot);
    }
    Ok(sessions)
}

fn scan_recent_codex_sessions() -> Result<Vec<ObservedSession>, String> {
    let root = codex_sessions_dir()?;
    if !root.exists() {
        return Ok(vec![]);
    }

    let now = SystemTime::now();
    let mut rollout_files = Vec::<(SystemTime, PathBuf)>::new();
    collect_files_with_extension(&root, "jsonl", &mut rollout_files)?;
    rollout_files.sort_by(|a, b| b.0.cmp(&a.0));

    let mut sessions = Vec::new();

    for (modified_at, path) in rollout_files.into_iter().take(MAX_RECENT_CODEX_ROLLOUTS) {
        let Ok(age) = now.duration_since(modified_at) else {
            continue;
        };
        if age > RECENT_SESSION_MAX_AGE {
            continue;
        }

        if let Some(snapshot) = parse_codex_rollout(&path) {
            sessions.push(snapshot);
        }
    }

    Ok(sessions)
}

fn scan_recent_gemini_sessions() -> Result<Vec<ObservedSession>, String> {
    let tmp_base = home_dir()?.join(".gemini").join("tmp");
    if !tmp_base.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let now = SystemTime::now();

    for entry in fs::read_dir(&tmp_base).map_err(|err| err.to_string())? {
        let Ok(entry) = entry else { continue };
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        let cwd = fs::read_to_string(project_dir.join(".project_root"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let chats_dir = project_dir.join("chats");
        if !chats_dir.exists() {
            continue;
        }

        let Some((path, modified_at)) = find_most_recent_matching_file(
            &chats_dir,
            |file_name| file_name.starts_with("session-") && file_name.ends_with(".json"),
        )?
        else {
            continue;
        };

        let Ok(age) = now.duration_since(modified_at) else {
            continue;
        };
        if age > RECENT_SESSION_MAX_AGE {
            continue;
        }

        if let Some(snapshot) = parse_gemini_session(&path, cwd.clone()) {
            if seen.insert(snapshot.session_id.clone()) {
                results.push(snapshot);
            }
        }
    }

    Ok(results)
}

fn scan_recent_opencode_sessions() -> Result<Vec<ObservedSession>, String> {
    let db_path = home_dir()?
        .join(".local")
        .join("share")
        .join("opencode")
        .join("opencode.db");
    if !db_path.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|err| err.to_string())?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, COALESCE(directory, ''), time_updated
            FROM session
            WHERE time_archived IS NULL
            ORDER BY time_updated DESC
            LIMIT ?1
            "#,
        )
        .map_err(|err| err.to_string())?;

    let mut rows = stmt
        .query([MAX_RECENT_PROVIDER_SESSIONS as i64])
        .map_err(|err| err.to_string())?;

    let now = SystemTime::now();
    let mut results = Vec::new();
    while let Some(row) = rows.next().map_err(|err| err.to_string())? {
        let session_id: String = row.get(0).map_err(|err| err.to_string())?;
        let cwd: String = row.get(1).unwrap_or_default();
        let updated_at_ms: i64 = row.get(2).unwrap_or_default();
        let modified_at = system_time_from_millis(updated_at_ms);

        let Ok(age) = now.duration_since(modified_at) else {
            continue;
        };
        if age > RECENT_SESSION_MAX_AGE {
            continue;
        }

        if let Some(snapshot) = parse_opencode_session(
            &conn,
            &session_id,
            (!cwd.trim().is_empty()).then_some(cwd),
        )? {
            results.push(snapshot);
        }
    }

    Ok(results)
}

fn parse_codex_rollout(path: &Path) -> Option<ObservedSession> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut state = CodexRolloutState::default();

    for line in reader.lines() {
        let Ok(line) = line else { continue };
        if line.trim().is_empty() {
            continue;
        }

        let Ok(json) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        apply_codex_rollout_line(&mut state, &json);
    }

    let session_id = state.session_id?;
    let summary = if state.current_turn_active {
        Some(
            state
                .session_title
                .clone()
                .unwrap_or_else(|| "等待 Codex 操作".to_string()),
        )
    } else {
        state.last_completion_summary.clone()
    };

    let event = if state.current_turn_active {
        EventType::Thinking
    } else {
        EventType::Complete
    };

    Some(ObservedSession {
        agent: AgentType::Codex,
        session_id,
        event,
        tool_name: Some("codex".to_string()),
        summary,
        working_directory: state.working_directory,
        session_title: state.session_title,
    })
}

fn apply_codex_rollout_line(state: &mut CodexRolloutState, json: &Value) {
    match json["type"].as_str() {
        Some("session_meta") => {
            state.session_id = json["payload"]["id"].as_str().map(str::to_string);
            state.working_directory = json["payload"]["cwd"].as_str().map(str::to_string);
        }
        Some("event_msg") => match json["payload"]["type"].as_str() {
            Some("user_message") => {
                let message = json["payload"]["message"].as_str().unwrap_or_default();
                if let Some(title) = extract_session_title(message) {
                    state.session_title = Some(title);
                }
            }
            Some("task_started") => {
                state.current_turn_active = true;
                state.last_completion_summary = None;
            }
            Some("task_complete") => {
                state.current_turn_active = false;
                state.last_completion_summary = json["payload"]["last_agent_message"]
                    .as_str()
                    .map(truncate)
                    .or_else(|| state.session_title.clone());
            }
            _ => {}
        },
        _ => {}
    }
}

fn parse_gemini_session(path: &Path, cwd: Option<String>) -> Option<ObservedSession> {
    let data = fs::read(path).ok()?;
    let json = serde_json::from_slice::<Value>(&data).ok()?;
    let session_id = json["sessionId"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string()
        });

    let messages = json["messages"].as_array().cloned().unwrap_or_default();
    let mut last_user = None;
    let mut last_assistant = None;

    for message in messages {
        let message_type = message["type"].as_str().unwrap_or_default().to_lowercase();
        let text = extract_message_text(&message["content"]);
        let Some(text) = text.filter(|value| !value.is_empty()) else {
            continue;
        };

        if message_type == "user" {
            last_user = Some(text);
        } else {
            last_assistant = Some(text);
        }
    }

    let session_title = last_user.as_deref().and_then(first_non_empty_line).map(truncate);
    let summary = session_title
        .clone()
        .or_else(|| last_assistant.as_ref().map(truncate))
        .or(Some("等待 Gemini 操作".to_string()));

    Some(ObservedSession {
        agent: AgentType::Gemini,
        session_id,
        event: EventType::Thinking,
        tool_name: Some("gemini".to_string()),
        summary,
        working_directory: cwd,
        session_title,
    })
}

fn parse_opencode_session(
    conn: &Connection,
    session_id: &str,
    cwd: Option<String>,
) -> Result<Option<ObservedSession>, String> {
    let mut message_stmt = conn
        .prepare(
            r#"
            SELECT p.message_id, COALESCE(json_extract(m.data, '$.role'), ''), p.time_created, p.data
            FROM part p
            JOIN message m ON m.id = p.message_id
            WHERE p.session_id = ?1
            ORDER BY p.time_created DESC
            LIMIT 80
            "#,
        )
        .map_err(|err| err.to_string())?;

    let mut rows = message_stmt
        .query([session_id])
        .map_err(|err| err.to_string())?;

    let mut seen_message_ids = HashSet::new();
    let mut combined: Vec<(i64, bool, String)> = Vec::new();

    while let Some(row) = rows.next().map_err(|err| err.to_string())? {
        let message_id: String = row.get(0).map_err(|err| err.to_string())?;
        if !seen_message_ids.insert(message_id) {
            continue;
        }
        let role: String = row.get(1).unwrap_or_default();
        let created_at: i64 = row.get(2).unwrap_or_default();
        let data: String = row.get(3).unwrap_or_default();
        let Ok(json) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        if json["type"].as_str() != Some("text") {
            continue;
        }
        let Some(text) = json["text"].as_str().map(str::trim).filter(|value| !value.is_empty()) else {
            continue;
        };
        match role.as_str() {
            "user" => combined.push((created_at, true, text.to_string())),
            "assistant" => combined.push((created_at, false, text.to_string())),
            _ => {}
        }
    }

    combined.sort_by_key(|entry| entry.0);

    let session_title = combined
        .iter()
        .rev()
        .find_map(|(_, is_user, text)| (*is_user).then(|| truncate(text)));
    let summary = combined
        .iter()
        .rev()
        .find_map(|(_, is_user, text)| (!*is_user).then(|| truncate(text)))
        .or_else(|| session_title.clone())
        .or(Some("等待 OpenCode 操作".to_string()));

    Ok(Some(ObservedSession {
        agent: AgentType::OpenCode,
        session_id: session_id.to_string(),
        event: EventType::Thinking,
        tool_name: Some("opencode".to_string()),
        summary,
        working_directory: cwd,
        session_title,
    }))
}

fn collect_files_with_extension(
    dir: &Path,
    extension: &str,
    files: &mut Vec<(SystemTime, PathBuf)>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|err| err.to_string())?;

        if metadata.is_dir() {
            collect_files_with_extension(&path, extension, files)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some(extension) {
            continue;
        }

        let modified_at = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        files.push((modified_at, path));
    }

    Ok(())
}

fn find_most_recent_matching_file(
    dir: &Path,
    matcher: impl Fn(&str) -> bool,
) -> Result<Option<(PathBuf, SystemTime)>, String> {
    let mut best: Option<(PathBuf, SystemTime)> = None;
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !matcher(file_name) {
            continue;
        }

        let modified_at = entry
            .metadata()
            .map_err(|err| err.to_string())?
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH);

        if best
            .as_ref()
            .map(|(_, best_modified)| modified_at > *best_modified)
            .unwrap_or(true)
        {
            best = Some((path, modified_at));
        }
    }
    Ok(best)
}

fn extract_session_title(message: &str) -> Option<String> {
    let request_marker = "## My request for Codex:";
    if let Some((_, tail)) = message.split_once(request_marker) {
        return first_non_empty_line(tail).map(truncate);
    }

    first_non_empty_line(message).map(truncate)
}

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn extract_message_text(content: &Value) -> Option<String> {
    if let Some(text) = content.as_str() {
        return Some(text.trim().to_string());
    }
    if let Some(items) = content.as_array() {
        for item in items {
            if let Some(text) = item.get("text").and_then(Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            if let Some(text) = item.get("content").and_then(Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

fn truncate(value: impl AsRef<str>) -> String {
    let value = value.as_ref().trim();
    if value.chars().count() > 180 {
        value.chars().take(180).collect::<String>() + "..."
    } else {
        value.to_string()
    }
}

fn system_time_from_millis(timestamp_ms: i64) -> SystemTime {
    if timestamp_ms <= 0 {
        return SystemTime::UNIX_EPOCH;
    }
    SystemTime::UNIX_EPOCH + Duration::from_millis(timestamp_ms as u64)
}

fn codex_sessions_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".codex").join("sessions"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        apply_codex_rollout_line, extract_message_text, extract_session_title, first_non_empty_line,
        CodexRolloutState, EventType,
    };

    #[test]
    fn extracts_request_title_from_ide_context_message() {
        let message = "# Context from my IDE setup:\n\n## My request for Codex:\n帮我修一下\n";

        assert_eq!(
            extract_session_title(message).as_deref(),
            Some("帮我修一下")
        );
    }

    #[test]
    fn rollout_state_tracks_active_and_complete_turns() {
        let mut state = CodexRolloutState::default();

        apply_codex_rollout_line(
            &mut state,
            &json!({
                "type": "session_meta",
                "payload": {
                    "id": "thread-1",
                    "cwd": "/tmp/demo"
                }
            }),
        );
        apply_codex_rollout_line(
            &mut state,
            &json!({
                "type": "event_msg",
                "payload": {
                    "type": "user_message",
                    "message": "## My request for Codex:\n修 bug"
                }
            }),
        );
        apply_codex_rollout_line(
            &mut state,
            &json!({
                "type": "event_msg",
                "payload": {
                    "type": "task_started"
                }
            }),
        );

        assert_eq!(state.session_id.as_deref(), Some("thread-1"));
        assert_eq!(state.working_directory.as_deref(), Some("/tmp/demo"));
        assert_eq!(state.session_title.as_deref(), Some("修 bug"));
        assert!(state.current_turn_active);

        apply_codex_rollout_line(
            &mut state,
            &json!({
                "type": "event_msg",
                "payload": {
                    "type": "task_complete",
                    "last_agent_message": "已经修好了"
                }
            }),
        );

        let event = if state.current_turn_active {
            EventType::Thinking
        } else {
            EventType::Complete
        };
        assert_eq!(event, EventType::Complete);
        assert_eq!(state.last_completion_summary.as_deref(), Some("已经修好了"));
    }

    #[test]
    fn extracts_text_from_rich_message_arrays() {
        let content = json!([
            { "type": "thinking", "text": "" },
            { "type": "text", "text": "Gemini says hello" }
        ]);

        assert_eq!(
            extract_message_text(&content).as_deref(),
            Some("Gemini says hello")
        );
    }

    #[test]
    fn first_non_empty_line_skips_blank_lines() {
        assert_eq!(
            first_non_empty_line("\n\n  hi there  \nsecond line").as_deref(),
            Some("hi there")
        );
    }
}
