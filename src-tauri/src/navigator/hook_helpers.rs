use std::{fs, path::PathBuf};

use serde_json::{json, Value};

pub const SOURCE_MARKER: &str = "copiwaifu";

// ── Path helpers ──────────────────────────────────────────────────────────────

pub fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

pub fn runtime_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".copiwaifu"))
}

pub fn hook_dir() -> Result<PathBuf, String> {
    Ok(runtime_dir()?.join("hooks"))
}

pub fn claude_settings_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".claude").join("settings.json"))
}

pub fn copilot_settings_path() -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".config")
        .join("github-copilot")
        .join("config.json"))
}

pub fn codex_config_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".codex").join("config.toml"))
}

pub fn backup_path() -> Result<PathBuf, String> {
    Ok(hook_dir()?.join("original-hooks.json"))
}

// ── JSON helpers ──────────────────────────────────────────────────────────────

pub fn read_json_or_default(path: &std::path::Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn write_json(path: &std::path::Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, body).map_err(|e| e.to_string())
}

// ── Command builders ──────────────────────────────────────────────────────────

pub fn hook_command(script: &std::path::Path, agent: &str, event: &str) -> String {
    format!(
        "node \"{}\" --agent {} --event {}",
        script.display(),
        agent,
        event
    )
}

// ── Claude-specific helpers ───────────────────────────────────────────────────

pub fn claude_hook_obj(command: &str) -> Value {
    json!({ "type": "command", "command": command })
}

pub fn cmd_has_marker(v: &Value) -> bool {
    v.get("command")
        .and_then(Value::as_str)
        .map(|c| c.contains(SOURCE_MARKER))
        .unwrap_or(false)
}

// ── TOML helpers (no toml crate) ──────────────────────────────────────────────

pub fn toml_find_notify(content: &str) -> Option<&str> {
    content
        .lines()
        .find(|l| l.trim_start().starts_with("notify = ["))
}

pub fn toml_parse_array(line: &str) -> Vec<String> {
    let start = line.find('[').unwrap_or(0) + 1;
    let end = line.rfind(']').unwrap_or(line.len());
    line[start..end]
        .split(',')
        .filter_map(|s| {
            let t = s.trim().trim_matches('"');
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        })
        .collect()
}

pub fn toml_build_notify(args: &[String]) -> String {
    let items: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
    format!("notify = [{}]", items.join(", "))
}

pub fn toml_upsert_notify(content: &str, new_line: &str) -> String {
    if content
        .lines()
        .any(|l| l.trim_start().starts_with("notify = ["))
    {
        content
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("notify = [") {
                    new_line.to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else if content.is_empty() {
        new_line.to_string()
    } else {
        format!("{}\n{}", content.trim_end(), new_line)
    }
}

pub fn toml_remove_notify(content: &str) -> String {
    content
        .lines()
        .filter(|l| !l.trim_start().starts_with("notify = ["))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Backup ────────────────────────────────────────────────────────────────────

pub fn backup_existing_hooks() -> Result<(), String> {
    let mut backup = json!({ "claude-code": {}, "copilot": {}, "codex": {} });

    if let Ok(root) = read_json_or_default(&claude_settings_path()?) {
        if let Some(hooks) = root.get("hooks").and_then(Value::as_object) {
            for (event, entries) in hooks {
                if let Some(arr) = entries.as_array() {
                    for outer in arr {
                        if let Some(inner_hooks) = outer.get("hooks").and_then(Value::as_array) {
                            for h in inner_hooks {
                                if !cmd_has_marker(h) {
                                    if let Some(cmd) = h.get("command").and_then(Value::as_str) {
                                        backup["claude-code"][event] = json!({ "command": cmd });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(root) = read_json_or_default(&copilot_settings_path()?) {
        if let Some(hooks) = root.get("hooks").and_then(Value::as_object) {
            for (event, entries) in hooks {
                if let Some(arr) = entries.as_array() {
                    for entry in arr {
                        let is_ours =
                            entry.get("source").and_then(Value::as_str) == Some(SOURCE_MARKER);
                        if !is_ours {
                            if let Some(cmd) = entry.get("command").and_then(Value::as_str) {
                                backup["copilot"][event] = json!({ "command": cmd });
                            }
                        }
                    }
                }
            }
        }
    }

    let codex_path = codex_config_path()?;
    if codex_path.exists() {
        let content = fs::read_to_string(&codex_path).unwrap_or_default();
        if let Some(line) = toml_find_notify(&content) {
            let args = toml_parse_array(line);
            if !args.iter().any(|a| a.contains(SOURCE_MARKER)) && !args.is_empty() {
                let arr: Vec<Value> = args.into_iter().map(|a| json!(a)).collect();
                backup["codex"]["notify"] = Value::Array(arr);
            }
        }
    }

    write_json(&backup_path()?, &backup)
}
