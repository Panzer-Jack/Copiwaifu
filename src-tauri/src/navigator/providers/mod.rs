mod claude;
mod codex;
mod copilot;
mod gemini;
mod opencode;

use super::events::{AgentType, EventType};

pub fn parse_agent_type(value: &str) -> Option<AgentType> {
    match value {
        "claude-code" | "claude_code" | "claude" => Some(AgentType::ClaudeCode),
        "copilot" | "copilot-cli" | "copilot_cli" => Some(AgentType::Copilot),
        "codex" | "codex-cli" | "codex_cli" => Some(AgentType::Codex),
        "gemini" | "gemini-cli" | "gemini_cli" => Some(AgentType::Gemini),
        "opencode" | "open-code" | "open_code" => Some(AgentType::OpenCode),
        _ => None,
    }
}

pub fn normalize_event(agent: AgentType, raw_event: &str) -> Result<EventType, String> {
    let normalized = match agent {
        AgentType::ClaudeCode => claude::normalize(raw_event),
        AgentType::Copilot => copilot::normalize(raw_event),
        AgentType::Codex => codex::normalize(raw_event),
        AgentType::Gemini => gemini::normalize(raw_event),
        AgentType::OpenCode => opencode::normalize(raw_event),
    };

    normalized.ok_or_else(|| format!("unsupported event for {}: {raw_event}", agent.as_str()))
}

fn normalize_shared(raw_event: &str) -> Option<EventType> {
    match raw_event {
        "session_start" | "SessionStart" | "sessionStart" => Some(EventType::SessionStart),
        "session_end" | "SessionEnd" | "sessionEnd" => Some(EventType::SessionEnd),
        "thinking" | "UserPromptSubmit" | "userPromptSubmitted" => Some(EventType::Thinking),
        "tool_use" | "PreToolUse" | "preToolUse" | "BeforeTool" | "BeforeAgent" | "before_tool"
        | "subagent_started" => Some(EventType::ToolUse),
        "tool_result" | "PostToolUse" | "postToolUse" | "AfterTool" | "AfterAgent"
        | "tool_finished" | "subagent_stopped" => Some(EventType::ToolResult),
        "error" | "PostToolUseFailure" | "StopFailure" | "errorOccurred" | "tool_error" => {
            Some(EventType::Error)
        }
        "complete" | "Stop" | "agentStop" | "turn_completed" => Some(EventType::Complete),
        "permission_request" | "PermissionRequest" | "needs_attention" | "question_requested"
        | "Notification" | "notification" => Some(EventType::NeedsAttention),
        _ => None,
    }
}

mod shared {
    use super::{normalize_shared, EventType};

    pub fn normalize(raw_event: &str) -> Option<EventType> {
        normalize_shared(raw_event)
    }
}
