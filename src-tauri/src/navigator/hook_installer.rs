use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

const SOURCE_MARKER: &str = "copiwaifu";
const CLAUDE_HOOK: &str = include_str!("../../../hooks/claude-hook.js");
const COPILOT_HOOK: &str = include_str!("../../../hooks/copilot-hook.js");
const COPIWAIFU_HOOK_DIR_MARKER: &str = ".copiwaifu/hooks/";

pub fn install_hooks() -> Result<(), String> {
    let hook_dir = hook_dir()?;
    fs::create_dir_all(&hook_dir).map_err(|err| err.to_string())?;

    let claude_hook_path = hook_dir.join("claude-hook.js");
    let copilot_hook_path = hook_dir.join("copilot-hook.js");

    write_hook_file(&claude_hook_path, CLAUDE_HOOK)?;
    write_hook_file(&copilot_hook_path, COPILOT_HOOK)?;

    install_claude_hooks(&claude_hook_path)?;
    install_copilot_hooks(&copilot_hook_path)?;
    Ok(())
}

pub fn uninstall_hooks() -> Result<(), String> {
    remove_hook_entries(&claude_settings_path()?)?;
    remove_hook_entries(&copilot_settings_path()?)?;

    let hook_dir = hook_dir()?;
    if hook_dir.exists() {
        fs::remove_dir_all(hook_dir).map_err(|err| err.to_string())?;
    }

    let port_file = runtime_dir()?.join("port");
    let _ = fs::remove_file(port_file);
    let _ = fs::remove_file("/tmp/copiwaifu-port");
    Ok(())
}

fn install_claude_hooks(script_path: &Path) -> Result<(), String> {
    let config_path = claude_settings_path()?;
    let hooks = [
        ("SessionStart", command_for(script_path, "session_start")),
        ("SessionEnd", command_for(script_path, "session_end")),
        ("UserPromptSubmit", command_for(script_path, "thinking")),
        ("PreToolUse", command_for(script_path, "tool_use")),
        ("PostToolUse", command_for(script_path, "tool_result")),
        ("PostToolUseFailure", command_for(script_path, "error")),
        ("Stop", command_for(script_path, "complete")),
        ("Notification", command_for(script_path, "complete")),
    ];

    upsert_claude_hooks(config_path.as_path(), &hooks)
}

fn install_copilot_hooks(script_path: &Path) -> Result<(), String> {
    let config_path = copilot_settings_path()?;
    let hooks = [
        ("sessionStart", command_for(script_path, "session_start")),
        ("sessionEnd", command_for(script_path, "session_end")),
        ("userPromptSubmitted", command_for(script_path, "thinking")),
        ("preToolUse", command_for(script_path, "tool_use")),
        ("postToolUse", command_for(script_path, "tool_result")),
        ("errorOccurred", command_for(script_path, "error")),
        ("agentStop", command_for(script_path, "complete")),
    ];

    upsert_hooks(config_path.as_path(), &hooks)
}

fn upsert_hooks(config_path: &Path, hooks: &[(&str, String)]) -> Result<(), String> {
    let mut root = read_json_or_default(config_path)?;
    if !root.is_object() {
        root = json!({});
    }

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| "hook config is not an object".to_string())?;
    let hooks_value = root_obj.entry("hooks").or_insert_with(|| json!({}));
    if !hooks_value.is_object() {
        *hooks_value = json!({});
    }

    let hooks_object = hooks_value
        .as_object_mut()
        .ok_or_else(|| "hooks field is not an object".to_string())?;

    for (event, command) in hooks {
        let entries = hooks_object
            .entry((*event).to_string())
            .or_insert_with(|| json!([]));
        if !entries.is_array() {
            *entries = json!([]);
        }

        let array = entries
            .as_array_mut()
            .ok_or_else(|| "hook entry is not an array".to_string())?;

        let mut updated = false;
        for value in array.iter_mut() {
            if value.get("source").and_then(Value::as_str) != Some(SOURCE_MARKER) {
                continue;
            }
            *value = json!({
                "command": command,
                "source": SOURCE_MARKER,
            });
            updated = true;
        }

        if !updated {
            array.push(json!({
                "command": command,
                "source": SOURCE_MARKER,
            }));
        }
    }

    write_json(config_path, &root)
}

fn upsert_claude_hooks(config_path: &Path, hooks: &[(&str, String)]) -> Result<(), String> {
    let mut root = read_json_or_default(config_path)?;
    if !root.is_object() {
        root = json!({});
    }

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| "hook config is not an object".to_string())?;
    let hooks_value = root_obj.entry("hooks").or_insert_with(|| json!({}));
    if !hooks_value.is_object() {
        *hooks_value = json!({});
    }

    let hooks_object = hooks_value
        .as_object_mut()
        .ok_or_else(|| "hooks field is not an object".to_string())?;

    for (event, command) in hooks {
        let entries = hooks_object
            .entry((*event).to_string())
            .or_insert_with(|| json!([]));
        if !entries.is_array() {
            *entries = json!([]);
        }

        let array = entries
            .as_array_mut()
            .ok_or_else(|| "hook entry is not an array".to_string())?;

        let mut next_entries = Vec::new();
        let mut inserted = false;

        for mut entry in std::mem::take(array) {
            if entry_command_matches(&entry, COPIWAIFU_HOOK_DIR_MARKER) {
                continue;
            }

            if let Some(hooks_array) = entry.get_mut("hooks").and_then(Value::as_array_mut) {
                let mut next_hooks = Vec::new();

                for hook in std::mem::take(hooks_array) {
                    if hook_command_matches(&hook, COPIWAIFU_HOOK_DIR_MARKER) {
                        if !inserted {
                            next_hooks.push(claude_command_hook(command));
                            inserted = true;
                        }
                        continue;
                    }
                    next_hooks.push(hook);
                }

                if !next_hooks.is_empty() {
                    *hooks_array = next_hooks;
                    next_entries.push(entry);
                }
                continue;
            }

            next_entries.push(entry);
        }

        if !inserted {
            next_entries.push(json!({
                "matcher": "",
                "hooks": [claude_command_hook(command)],
            }));
        }

        *array = next_entries;
    }

    write_json(config_path, &root)
}

fn remove_hook_entries(config_path: &Path) -> Result<(), String> {
    if !config_path.exists() {
        return Ok(());
    }

    let mut root = read_json_or_default(config_path)?;
    let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        return Ok(());
    };

    for value in hooks.values_mut() {
        let Some(entries) = value.as_array_mut() else {
            continue;
        };

        let mut next_entries = Vec::new();
        for mut entry in std::mem::take(entries) {
            if entry.get("source").and_then(Value::as_str) == Some(SOURCE_MARKER)
                || entry_command_matches(&entry, COPIWAIFU_HOOK_DIR_MARKER)
            {
                continue;
            }

            if let Some(hooks_array) = entry.get_mut("hooks").and_then(Value::as_array_mut) {
                hooks_array.retain(|hook| !hook_command_matches(hook, COPIWAIFU_HOOK_DIR_MARKER));
                if hooks_array.is_empty() {
                    continue;
                }
            }

            next_entries.push(entry);
        }

        *entries = next_entries;
    }

    hooks.retain(|_, value| {
        value
            .as_array()
            .map(|entries| !entries.is_empty())
            .unwrap_or(true)
    });
    write_json(config_path, &root)
}

fn command_for(script_path: &Path, event_name: &str) -> String {
    format!("node \"{}\" {}", script_path.display(), event_name)
}

fn claude_command_hook(command: &str) -> Value {
    json!({
        "type": "command",
        "command": command,
    })
}

fn entry_command_matches(entry: &Value, marker: &str) -> bool {
    entry
        .get("command")
        .and_then(Value::as_str)
        .map(|command| command.contains(marker))
        .unwrap_or(false)
}

fn hook_command_matches(hook: &Value, marker: &str) -> bool {
    hook.get("command")
        .and_then(Value::as_str)
        .map(|command| command.contains(marker))
        .unwrap_or(false)
}

fn write_hook_file(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|err| err.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|err| err.to_string())?;
    }

    Ok(())
}

fn read_json_or_default(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }

    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let body = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(path, body).map_err(|err| err.to_string())
}

fn runtime_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".copiwaifu"))
}

fn hook_dir() -> Result<PathBuf, String> {
    Ok(runtime_dir()?.join("hooks"))
}

fn claude_settings_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".claude").join("settings.json"))
}

fn copilot_settings_path() -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".config")
        .join("github-copilot")
        .join("config.json"))
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}
