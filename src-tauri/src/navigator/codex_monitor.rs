use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime},
};

use serde_json::Value;
use tauri::AppHandle;

use super::{
    emit_all,
    events::{AgentEvent, AgentType, EventData, EventType},
    state::NavigatorState,
};

const ACTIVE_POLL_INTERVAL: Duration = Duration::from_millis(500);
const IDLE_POLL_INTERVAL: Duration = Duration::from_secs(2);
const IDLE_THRESHOLD: Duration = Duration::from_secs(30);

struct TrackedFile {
    offset: u64,
    partial: String,
    session_id: String,
    cwd: Option<String>,
    last_seen: Instant,
}

pub fn start(app_handle: AppHandle, state: Arc<Mutex<NavigatorState>>) {
    thread::spawn(move || {
        let Some(base_dir) = codex_session_dir() else {
            return;
        };

        if !base_dir.exists() {
            eprintln!("codex monitor skipped: {} not found", base_dir.display());
            return;
        }

        let mut tracked = HashMap::<PathBuf, TrackedFile>::new();
        let mut last_activity = Instant::now();

        loop {
            let had_activity = poll_once(&base_dir, &mut tracked, &app_handle, &state);
            if had_activity {
                last_activity = Instant::now();
            }

            tracked.retain(|_, file| file.last_seen.elapsed() < Duration::from_secs(180));

            let sleep_for = if last_activity.elapsed() > IDLE_THRESHOLD {
                IDLE_POLL_INTERVAL
            } else {
                ACTIVE_POLL_INTERVAL
            };
            thread::sleep(sleep_for);
        }
    });
}

fn poll_once(
    base_dir: &Path,
    tracked: &mut HashMap<PathBuf, TrackedFile>,
    app_handle: &AppHandle,
    state: &Arc<Mutex<NavigatorState>>,
) -> bool {
    let mut had_activity = false;

    for file_path in recent_rollout_files(base_dir) {
        let entry = tracked
            .entry(file_path.clone())
            .or_insert_with(|| TrackedFile {
                offset: 0,
                partial: String::new(),
                session_id: session_id_from_path(&file_path),
                cwd: None,
                last_seen: Instant::now(),
            });

        match process_file(file_path.as_path(), entry) {
            Ok(events) => {
                if !events.is_empty() {
                    had_activity = true;
                    for event in events {
                        let emissions = match state.lock() {
                            Ok(mut navigator) => navigator.apply_event(event),
                            Err(err) => {
                                eprintln!("codex monitor lock poisoned: {err}");
                                continue;
                            }
                        };
                        emit_all(app_handle, emissions);
                    }
                }
            }
            Err(err) => {
                eprintln!("codex monitor failed for {}: {err}", file_path.display());
            }
        }
    }

    had_activity
}

fn process_file(path: &Path, tracked: &mut TrackedFile) -> Result<Vec<AgentEvent>, String> {
    let metadata = fs::metadata(path).map_err(|err| err.to_string())?;
    let size = metadata.len();
    tracked.last_seen = Instant::now();

    if size < tracked.offset {
        tracked.offset = 0;
        tracked.partial.clear();
    }

    if size == tracked.offset {
        return Ok(Vec::new());
    }

    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let slice = bytes
        .get(tracked.offset as usize..)
        .ok_or_else(|| "invalid file offset".to_string())?;
    tracked.offset = size;

    let text = format!("{}{}", tracked.partial, String::from_utf8_lossy(slice));
    let mut lines = text.lines().map(str::to_string).collect::<Vec<_>>();
    tracked.partial = if text.ends_with('\n') {
        String::new()
    } else {
        lines.pop().unwrap_or_default()
    };

    let mut events = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(event) = parse_jsonl_line(&line, tracked)? {
            events.push(event);
        }
    }

    Ok(events)
}

fn parse_jsonl_line(line: &str, tracked: &mut TrackedFile) -> Result<Option<AgentEvent>, String> {
    let value = serde_json::from_str::<Value>(line).map_err(|err| err.to_string())?;
    let record_type = value
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing record type".to_string())?;

    let payload = value.get("payload").cloned().unwrap_or(Value::Null);
    let payload_type = payload.get("type").and_then(Value::as_str);

    if record_type == "session_meta" {
        if let Some(cwd) = payload.get("cwd").and_then(Value::as_str) {
            tracked.cwd = Some(cwd.to_string());
        }

        return Ok(Some(AgentEvent {
            agent: AgentType::Codex,
            session_id: tracked.session_id.clone(),
            event: EventType::SessionStart,
            data: EventData::default(),
        }));
    }

    let mapped = match (record_type, payload_type) {
        ("event_msg", Some("task_started")) => Some((EventType::Thinking, None, None)),
        ("event_msg", Some("user_message")) => Some((EventType::Thinking, None, None)),
        ("event_msg", Some("task_complete")) => Some((EventType::Complete, None, None)),
        ("event_msg", Some("turn_aborted")) => Some((EventType::Complete, None, None)),
        ("event_msg", Some("context_compacted")) => Some((
            EventType::ToolUse,
            Some("compact".to_string()),
            Some("整理上下文".to_string()),
        )),
        ("response_item", Some("function_call")) => {
            let tool_name = payload
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| Some("shell_command".to_string()));
            let summary = extract_function_summary(&payload);
            Some((EventType::ToolUse, tool_name, summary))
        }
        ("response_item", Some("custom_tool_call")) => {
            let tool_name = payload
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| Some("custom_tool".to_string()));
            Some((EventType::ToolUse, tool_name, tracked.cwd.clone()))
        }
        ("response_item", Some("web_search_call")) => Some((
            EventType::ToolUse,
            Some("web_search".to_string()),
            payload
                .get("query")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        _ => None,
    };

    let Some((event, tool_name, summary)) = mapped else {
        return Ok(None);
    };

    Ok(Some(AgentEvent {
        agent: AgentType::Codex,
        session_id: tracked.session_id.clone(),
        event,
        data: EventData {
            tool_name,
            summary,
            permission_id: None,
        },
    }))
}

fn extract_function_summary(payload: &Value) -> Option<String> {
    let arguments = payload.get("arguments")?;
    if arguments.is_object() {
        return arguments
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_string);
    }

    let raw = arguments.as_str()?;
    let parsed = serde_json::from_str::<Value>(raw).ok()?;
    parsed
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn recent_rollout_files(base_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let now = SystemTime::now();

    for year_entry in read_dir(base_dir) {
        for month_entry in read_dir(&year_entry.path()) {
            for day_entry in read_dir(&month_entry.path()) {
                let Ok(metadata) = day_entry.metadata() else {
                    continue;
                };
                let Ok(modified) = metadata.modified() else {
                    continue;
                };
                if now
                    .duration_since(modified)
                    .unwrap_or_else(|_| Duration::from_secs(0))
                    > Duration::from_secs(8 * 24 * 60 * 60)
                {
                    continue;
                }

                for file_entry in read_dir(&day_entry.path()) {
                    if file_entry.path().extension() != Some(OsStr::new("jsonl")) {
                        continue;
                    }
                    if !file_entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("rollout-")
                    {
                        continue;
                    }
                    files.push(file_entry.path());
                }
            }
        }
    }

    files.sort();
    files
}

fn read_dir(path: &Path) -> Vec<fs::DirEntry> {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .collect()
}

fn session_id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .map(|name| format!("codex:{name}"))
        .unwrap_or_else(|| "codex:unknown".to_string())
}

fn codex_session_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".codex").join("sessions"))
}
