use std::{fs, path::PathBuf};

use serde_json::{json, Value};

use crate::platform;

pub const SOURCE_MARKER: &str = "copiwaifu";

// ── Path helpers ──────────────────────────────────────────────────────────────

pub fn home_dir() -> Result<PathBuf, String> {
    platform::home_dir_result()
}

pub fn runtime_dir() -> Result<PathBuf, String> {
    platform::runtime_dir()
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

pub fn gemini_settings_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".gemini").join("settings.json"))
}

pub fn opencode_plugin_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".config").join("opencode").join("plugins"))
}

pub fn opencode_plugin_path() -> Result<PathBuf, String> {
    Ok(opencode_plugin_dir()?.join("copiwaifu.js"))
}

pub fn opencode_config_path() -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".config")
        .join("opencode")
        .join("config.json"))
}

pub fn opencode_config_path_new() -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".config")
        .join("opencode")
        .join("opencode.json"))
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

/// Find the top-level notify value span, handling both single-line and multi-line arrays.
/// Returns `Some((start_line_idx, end_line_idx))` (inclusive).
fn toml_notify_span(lines: &[&str]) -> Option<(usize, usize)> {
    let mut start = None;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            break;
        }
        if trimmed.starts_with("notify") {
            start = Some(idx);
            break;
        }
    }
    let start = start?;
    let first = lines[start];
    // Single-line: notify = [...] on one line
    if first.contains('[') && first.contains(']') {
        return Some((start, start));
    }
    // Multi-line: find the closing ']'
    for (i, line) in lines.iter().enumerate().skip(start) {
        if line.contains(']') {
            return Some((start, i));
        }
    }
    // Unclosed bracket — treat as single line to avoid eating the whole file
    Some((start, start))
}

/// Extract the full notify value text (may span multiple lines).
pub fn toml_find_notify(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let (start, end) = toml_notify_span(&lines)?;
    Some(lines[start..=end].join("\n"))
}

pub fn toml_parse_array(text: &str) -> Vec<String> {
    let start = text.find('[').unwrap_or(0) + 1;
    let end = text.rfind(']').unwrap_or(text.len());
    text[start..end]
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
    let items: Vec<String> = args
        .iter()
        .map(|a| serde_json::to_string(a).unwrap_or_else(|_| format!("{a:?}")))
        .collect();
    format!("notify = [{}]", items.join(", "))
}

pub fn toml_upsert_notify(content: &str, new_line: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if let Some((start, end)) = toml_notify_span(&lines) {
        let mut result: Vec<String> = Vec::with_capacity(lines.len());
        result.extend(lines[..start].iter().map(|l| l.to_string()));
        result.push(new_line.to_string());
        result.extend(lines[end + 1..].iter().map(|l| l.to_string()));
        result.join("\n")
    } else if content.is_empty() {
        new_line.to_string()
    } else {
        let insert_at = lines
            .iter()
            .position(|line| line.trim_start().starts_with('['))
            .unwrap_or(lines.len());
        let mut result: Vec<String> = Vec::with_capacity(lines.len() + 1);
        result.extend(lines[..insert_at].iter().map(|l| l.to_string()));
        result.push(new_line.to_string());
        result.extend(lines[insert_at..].iter().map(|l| l.to_string()));
        result.join("\n")
    }
}

pub fn toml_remove_notify(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if let Some((start, end)) = toml_notify_span(&lines) {
        let mut result: Vec<String> = Vec::with_capacity(lines.len());
        result.extend(lines[..start].iter().map(|l| l.to_string()));
        result.extend(lines[end + 1..].iter().map(|l| l.to_string()));
        result.join("\n")
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{toml_build_notify, toml_find_notify, toml_remove_notify, toml_upsert_notify};

    #[test]
    fn upsert_notify_inserts_before_first_table() {
        let content = "model = \"gpt-5\"\n\n[projects.\"/tmp\"]\ntrust_level = \"trusted\"";
        let updated = toml_upsert_notify(content, "notify = [\"node\", \"hook.js\"]");

        assert_eq!(
            updated,
            "model = \"gpt-5\"\n\nnotify = [\"node\", \"hook.js\"]\n[projects.\"/tmp\"]\ntrust_level = \"trusted\""
        );
    }

    #[test]
    fn notify_helpers_ignore_nested_notify_keys() {
        let content = "[profiles.default]\nnotify = [\"nested\"]\n";

        assert!(toml_find_notify(content).is_none());
        assert_eq!(toml_remove_notify(content), content);
        assert_eq!(
            toml_upsert_notify(content, "notify = [\"node\", \"hook.js\"]"),
            "notify = [\"node\", \"hook.js\"]\n[profiles.default]\nnotify = [\"nested\"]"
        );
    }

    #[test]
    fn build_notify_escapes_windows_paths() {
        let line = toml_build_notify(&[
            "node".to_string(),
            r"C:\Users\name\.copiwaifu\hooks\copiwaifu-hook.js".to_string(),
        ]);

        assert_eq!(
            line,
            r#"notify = ["node", "C:\\Users\\name\\.copiwaifu\\hooks\\copiwaifu-hook.js"]"#
        );
    }
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
            let args = toml_parse_array(&line);
            if !args.iter().any(|a| a.contains(SOURCE_MARKER)) && !args.is_empty() {
                let arr: Vec<Value> = args.into_iter().map(|a| json!(a)).collect();
                backup["codex"]["notify"] = Value::Array(arr);
            }
        }
    }

    write_json(&backup_path()?, &backup)
}
