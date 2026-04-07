use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use serde_json::Value;
use tauri::AppHandle;

use super::{
    emit_all,
    events::{AgentEvent, AgentType, EventData, EventType},
    hook_helpers::home_dir,
    state::NavigatorState,
};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const RECENT_ROLLOUT_MAX_AGE: Duration = Duration::from_secs(90);
const MAX_RECENT_ROLLOUTS: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedCodexSession {
    session_id: String,
    event: EventType,
    summary: Option<String>,
    working_directory: Option<String>,
    session_title: Option<String>,
}

impl ObservedCodexSession {
    fn into_agent_event(self) -> AgentEvent {
        AgentEvent {
            agent: AgentType::Codex,
            session_id: self.session_id,
            event: self.event,
            data: EventData {
                tool_name: Some("codex".to_string()),
                summary: self.summary,
                working_directory: self.working_directory,
                session_title: self.session_title,
                needs_attention: Some(false),
            },
        }
    }
}

#[derive(Default)]
struct RolloutParseState {
    session_id: Option<String>,
    working_directory: Option<String>,
    session_title: Option<String>,
    current_turn_active: bool,
    last_completion_summary: Option<String>,
}

pub fn start(app_handle: AppHandle, state: Arc<Mutex<NavigatorState>>) {
    thread::spawn(move || {
        let mut previous = HashMap::<String, ObservedCodexSession>::new();

        loop {
            thread::sleep(POLL_INTERVAL);

            let observed = scan_recent_codex_sessions().unwrap_or_default();
            let changed: Vec<ObservedCodexSession> = observed
                .iter()
                .filter_map(|(session_id, snapshot)| match previous.get(session_id) {
                    Some(last) if last == snapshot => None,
                    _ => Some(snapshot.clone()),
                })
                .collect();

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
                    eprintln!("codex activity lock poisoned: {err}");
                    continue;
                }
            };

            emit_all(&app_handle, emissions);
        }
    });
}

fn scan_recent_codex_sessions() -> Result<HashMap<String, ObservedCodexSession>, String> {
    let root = codex_sessions_dir()?;
    if !root.exists() {
        return Ok(HashMap::new());
    }

    let now = SystemTime::now();
    let mut rollout_files = Vec::<(SystemTime, PathBuf)>::new();
    collect_rollout_files(&root, &mut rollout_files)?;
    rollout_files.sort_by(|a, b| b.0.cmp(&a.0));

    let mut sessions = HashMap::new();

    for (modified_at, path) in rollout_files.into_iter().take(MAX_RECENT_ROLLOUTS) {
        let Ok(age) = now.duration_since(modified_at) else {
            continue;
        };
        if age > RECENT_ROLLOUT_MAX_AGE {
            continue;
        }

        if let Some(snapshot) = parse_rollout(&path) {
            sessions.insert(snapshot.session_id.clone(), snapshot);
        }
    }

    Ok(sessions)
}

fn collect_rollout_files(dir: &Path, files: &mut Vec<(SystemTime, PathBuf)>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|err| err.to_string())?;

        if metadata.is_dir() {
            collect_rollout_files(&path, files)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }

        let modified_at = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        files.push((modified_at, path));
    }

    Ok(())
}

fn parse_rollout(path: &Path) -> Option<ObservedCodexSession> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut state = RolloutParseState::default();

    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }

        let Ok(json) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        apply_rollout_line(&mut state, &json);
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

    Some(ObservedCodexSession {
        session_id,
        event,
        summary,
        working_directory: state.working_directory,
        session_title: state.session_title,
    })
}

fn apply_rollout_line(state: &mut RolloutParseState, json: &Value) {
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

fn truncate(value: impl AsRef<str>) -> String {
    let value = value.as_ref().trim();
    if value.chars().count() > 180 {
        value.chars().take(180).collect::<String>() + "..."
    } else {
        value.to_string()
    }
}

fn codex_sessions_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".codex").join("sessions"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{apply_rollout_line, extract_session_title, EventType, RolloutParseState};

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
        let mut state = RolloutParseState::default();

        apply_rollout_line(
            &mut state,
            &json!({
                "type": "session_meta",
                "payload": {
                    "id": "thread-1",
                    "cwd": "/tmp/demo"
                }
            }),
        );
        apply_rollout_line(
            &mut state,
            &json!({
                "type": "event_msg",
                "payload": {
                    "type": "user_message",
                    "message": "## My request for Codex:\n修 bug"
                }
            }),
        );
        apply_rollout_line(
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

        apply_rollout_line(
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
}
