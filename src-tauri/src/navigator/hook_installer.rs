use std::{fs, path::Path};

use serde_json::{json, Value};

use super::hook_helpers::{
    backup_existing_hooks, backup_path, claude_hook_obj, claude_settings_path, cmd_has_marker,
    codex_config_path, copilot_settings_path, gemini_settings_path, hook_command, hook_dir,
    opencode_config_path, opencode_config_path_new, opencode_plugin_dir, opencode_plugin_path,
    read_json_or_default, runtime_dir, toml_build_notify, toml_remove_notify,
    toml_upsert_notify, write_json, SOURCE_MARKER,
};

const COPIWAIFU_HOOK: &str = include_str!("../../../hooks/copiwaifu-hook.js");
const COPIWAIFU_OPENCODE_PLUGIN: &str = include_str!("../../../hooks/copiwaifu-opencode.js");

// ── Public API ────────────────────────────────────────────────────────────────

pub fn install_hooks() -> Result<(), String> {
    let dir = hook_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let script = dir.join("copiwaifu-hook.js");
    fs::write(&script, COPIWAIFU_HOOK).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }

    backup_existing_hooks()?;
    install_claude_hooks(&script)?;
    install_copilot_hooks(&script)?;
    install_codex_hooks(&script)?;
    install_gemini_hooks(&script)?;
    install_opencode_plugin()?;
    Ok(())
}

pub fn uninstall_hooks() -> Result<(), String> {
    remove_claude_hooks()?;
    remove_copilot_hooks()?;
    remove_codex_hooks()?;
    remove_gemini_hooks()?;
    remove_opencode_plugin()?;

    let dir = hook_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    let _ = fs::remove_file(runtime_dir()?.join("port"));
    let _ = fs::remove_file("/tmp/copiwaifu-port");
    Ok(())
}

// ── Claude ────────────────────────────────────────────────────────────────────

const CLAUDE_EVENTS: &[&str] = &[
    "SessionStart",
    "SessionEnd",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "Stop",
    "Notification",
    "PermissionRequest",
];

fn install_claude_hooks(script: &Path) -> Result<(), String> {
    let config = claude_settings_path()?;
    let mut root = read_json_or_default(&config)?;
    if !root.is_object() {
        root = json!({});
    }

    let hooks_map = root["hooks"]
        .as_object_mut()
        .map(|_| ())
        .unwrap_or_else(|| {
            root["hooks"] = json!({});
        });
    let _ = hooks_map;
    let hooks_obj = root["hooks"]
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    for &event in CLAUDE_EVENTS {
        let cmd = hook_command(script, "claude-code", event);
        let entries = hooks_obj.entry(event).or_insert_with(|| json!([]));
        if !entries.is_array() {
            *entries = json!([]);
        }
        let arr = entries.as_array_mut().ok_or("not an array")?;

        // Find existing copiwaifu outer entry and update its inner hook
        let mut found = false;
        for outer in arr.iter_mut() {
            if let Some(inner) = outer.get_mut("hooks").and_then(Value::as_array_mut) {
                if inner.iter().any(|h| cmd_has_marker(h)) {
                    // Replace our hook in-place
                    for h in inner.iter_mut() {
                        if cmd_has_marker(h) {
                            *h = claude_hook_obj(&cmd);
                        }
                    }
                    found = true;
                    break;
                }
            }
        }
        if !found {
            arr.push(json!({ "matcher": "", "hooks": [claude_hook_obj(&cmd)] }));
        }
    }

    write_json(&config, &root)
}

fn remove_claude_hooks() -> Result<(), String> {
    let config = claude_settings_path()?;
    if !config.exists() {
        return Ok(());
    }
    let mut root = read_json_or_default(&config)?;
    let Some(hooks_obj) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        return Ok(());
    };
    for entries in hooks_obj.values_mut() {
        let Some(arr) = entries.as_array_mut() else {
            continue;
        };
        arr.retain_mut(|outer| {
            let Some(inner) = outer.get_mut("hooks").and_then(Value::as_array_mut) else {
                return true;
            };
            inner.retain(|h| !cmd_has_marker(h));
            !inner.is_empty()
        });
    }
    hooks_obj.retain(|_, v| v.as_array().map(|a| !a.is_empty()).unwrap_or(true));
    write_json(&config, &root)
}

// ── Copilot ───────────────────────────────────────────────────────────────────

const COPILOT_EVENTS: &[&str] = &[
    "sessionStart",
    "sessionEnd",
    "userPromptSubmitted",
    "preToolUse",
    "postToolUse",
    "errorOccurred",
    "agentStop",
];

fn install_copilot_hooks(script: &Path) -> Result<(), String> {
    let config = copilot_settings_path()?;
    let mut root = read_json_or_default(&config)?;
    if !root.is_object() {
        root = json!({});
    }
    if root["hooks"].is_null() {
        root["hooks"] = json!({});
    }
    let hooks_obj = root["hooks"]
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    for &event in COPILOT_EVENTS {
        let cmd = hook_command(script, "copilot", event);
        let entries = hooks_obj.entry(event).or_insert_with(|| json!([]));
        if !entries.is_array() {
            *entries = json!([]);
        }
        let arr = entries.as_array_mut().ok_or("not an array")?;

        let mut found = false;
        for entry in arr.iter_mut() {
            if entry.get("source").and_then(Value::as_str) == Some(SOURCE_MARKER) {
                *entry = json!({ "command": cmd, "source": SOURCE_MARKER });
                found = true;
                break;
            }
        }
        if !found {
            arr.push(json!({ "command": cmd, "source": SOURCE_MARKER }));
        }
    }

    write_json(&config, &root)
}

fn remove_copilot_hooks() -> Result<(), String> {
    let config = copilot_settings_path()?;
    if !config.exists() {
        return Ok(());
    }
    let mut root = read_json_or_default(&config)?;
    let Some(hooks_obj) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        return Ok(());
    };
    for entries in hooks_obj.values_mut() {
        if let Some(arr) = entries.as_array_mut() {
            arr.retain(|e| e.get("source").and_then(Value::as_str) != Some(SOURCE_MARKER));
        }
    }
    hooks_obj.retain(|_, v| v.as_array().map(|a| !a.is_empty()).unwrap_or(true));
    write_json(&config, &root)
}

// ── Codex ─────────────────────────────────────────────────────────────────────

fn install_codex_hooks(script: &Path) -> Result<(), String> {
    let config = codex_config_path()?;
    if let Some(parent) = config.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = if config.exists() {
        fs::read_to_string(&config).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let node = which_node();
    let args = vec![
        node,
        script.to_string_lossy().to_string(),
        "--agent".to_string(),
        "codex".to_string(),
        "--event".to_string(),
        "notify".to_string(),
    ];
    let new_line = toml_build_notify(&args);
    let updated = toml_upsert_notify(&content, &new_line);
    fs::write(&config, updated).map_err(|e| e.to_string())
}

fn remove_codex_hooks() -> Result<(), String> {
    let config = codex_config_path()?;
    if !config.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&config).map_err(|e| e.to_string())?;

    // Restore from backup if available
    let backup = backup_path()?;
    if backup.exists() {
        if let Ok(bk) = read_json_or_default(&backup) {
            if let Some(arr) = bk["codex"]["notify"].as_array() {
                let args: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                if !args.is_empty() {
                    let restored = toml_upsert_notify(&content, &toml_build_notify(&args));
                    return fs::write(&config, restored).map_err(|e| e.to_string());
                }
            }
        }
    }

    let cleaned = toml_remove_notify(&content);
    fs::write(&config, cleaned).map_err(|e| e.to_string())
}

// ── Gemini ────────────────────────────────────────────────────────────────────

const GEMINI_EVENTS: &[&str] = &[
    "SessionStart",
    "SessionEnd",
    "BeforeTool",
    "AfterTool",
    "BeforeAgent",
    "AfterAgent",
];

fn install_gemini_hooks(script: &Path) -> Result<(), String> {
    let config = gemini_settings_path()?;
    if !config
        .parent()
        .map(Path::exists)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let mut root = read_json_or_default(&config)?;
    if !root.is_object() {
        root = json!({});
    }
    if root["hooks"].is_null() {
        root["hooks"] = json!({});
    }
    let hooks_obj = root["hooks"]
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    for &event in GEMINI_EVENTS {
        let cmd = hook_command(script, "gemini", event);
        let entries = hooks_obj.entry(event).or_insert_with(|| json!([]));
        if !entries.is_array() {
            *entries = json!([]);
        }
        let arr = entries.as_array_mut().ok_or("not an array")?;

        arr.retain(|entry| {
            let Some(inner) = entry.get("hooks").and_then(Value::as_array) else {
                return true;
            };
            !inner.iter().any(cmd_has_marker)
        });

        arr.push(json!({
            "hooks": [{
                "type": "command",
                "command": cmd,
                "timeout": 5000
            }]
        }));
    }

    write_json(&config, &root)
}

fn remove_gemini_hooks() -> Result<(), String> {
    let config = gemini_settings_path()?;
    if !config.exists() {
        return Ok(());
    }
    let mut root = read_json_or_default(&config)?;
    let Some(hooks_obj) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        return Ok(());
    };

    for entries in hooks_obj.values_mut() {
        let Some(arr) = entries.as_array_mut() else {
            continue;
        };
        arr.retain(|entry| {
            let Some(inner) = entry.get("hooks").and_then(Value::as_array) else {
                return true;
            };
            !inner.iter().any(cmd_has_marker)
        });
    }

    hooks_obj.retain(|_, value| value.as_array().map(|arr| !arr.is_empty()).unwrap_or(true));
    write_json(&config, &root)
}

// ── OpenCode ──────────────────────────────────────────────────────────────────

fn install_opencode_plugin() -> Result<(), String> {
    let config_path = opencode_config_path()?;
    let config_path_new = opencode_config_path_new()?;
    let config_dir = config_path
        .parent()
        .ok_or_else(|| "missing opencode config dir".to_string())?;

    if !config_dir.exists() {
        return Ok(());
    }

    let plugin_dir = opencode_plugin_dir()?;
    fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    let plugin_path = opencode_plugin_path()?;
    fs::write(&plugin_path, COPIWAIFU_OPENCODE_PLUGIN).map_err(|e| e.to_string())?;

    register_opencode_plugin(&config_path_new, &plugin_path)?;
    cleanup_opencode_plugin_registration(&config_path)?;
    Ok(())
}

fn remove_opencode_plugin() -> Result<(), String> {
    let plugin_path = opencode_plugin_path()?;
    if plugin_path.exists() {
        let _ = fs::remove_file(&plugin_path);
    }

    cleanup_opencode_plugin_registration(&opencode_config_path_new()?)?;
    cleanup_opencode_plugin_registration(&opencode_config_path()?)?;
    Ok(())
}

fn register_opencode_plugin(config_path: &Path, plugin_path: &Path) -> Result<(), String> {
    let plugin_ref = format!("file://{}", plugin_path.display());
    let mut root = read_json_or_default(config_path)?;
    if !root.is_object() {
        root = json!({});
    }

    let mut plugins = root["plugin"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    plugins.retain(|entry| {
        !entry
            .as_str()
            .map(|value| value.contains(SOURCE_MARKER))
            .unwrap_or(false)
    });
    plugins.push(Value::String(plugin_ref));
    root["plugin"] = Value::Array(plugins);

    if root.get("$schema").is_none() {
        root["$schema"] = Value::String("https://opencode.ai/config.json".to_string());
    }

    write_json(config_path, &root)
}

fn cleanup_opencode_plugin_registration(config_path: &Path) -> Result<(), String> {
    if !config_path.exists() {
        return Ok(());
    }
    let mut root = read_json_or_default(config_path)?;
    let Some(plugins) = root.get_mut("plugin").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    plugins.retain(|entry| {
        !entry
            .as_str()
            .map(|value| value.contains(SOURCE_MARKER))
            .unwrap_or(false)
    });
    if plugins.is_empty() {
        root.as_object_mut().map(|obj| obj.remove("plugin"));
    }
    write_json(config_path, &root)
}

fn which_node() -> String {
    let candidates = ["/opt/homebrew/bin/node", "/usr/local/bin/node", "node"];
    for c in &candidates {
        if Path::new(c).exists() {
            return c.to_string();
        }
    }
    "node".to_string()
}
