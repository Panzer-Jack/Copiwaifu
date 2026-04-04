use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    ClaudeCode,
    Copilot,
    Codex,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Copilot => "copilot",
            Self::Codex => "codex",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStart,
    SessionEnd,
    Thinking,
    ToolUse,
    ToolResult,
    Error,
    PermissionRequest,
    Complete,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Thinking,
    ToolUse,
    Error,
    WaitingPermission,
}

impl AgentState {
    pub fn priority(&self) -> u8 {
        match self {
            Self::WaitingPermission => 5,
            Self::Error => 4,
            Self::ToolUse => 3,
            Self::Thinking => 2,
            Self::Idle => 1,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventData {
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub permission_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentEvent {
    pub agent: AgentType,
    pub session_id: String,
    pub event: EventType,
    #[serde(default)]
    pub data: EventData,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IncomingHookEvent {
    #[serde(default)]
    pub agent: Option<AgentType>,
    #[serde(default, alias = "agent_id")]
    pub agent_id: Option<String>,
    pub session_id: String,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub state: Option<AgentState>,
    #[serde(default)]
    pub data: EventData,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub permission_id: Option<String>,
}

impl IncomingHookEvent {
    pub fn into_agent_event(self) -> Result<AgentEvent, String> {
        let agent = self
            .agent
            .or_else(|| self.agent_id.as_deref().and_then(parse_agent_type))
            .ok_or_else(|| "missing agent".to_string())?;

        let event = if let Some(raw_event) = self.event.as_deref() {
            parse_event_type(raw_event)?
        } else if let Some(state) = self.state {
            event_type_from_state(state)
        } else {
            return Err("missing event".to_string());
        };

        let mut data = self.data;
        if data.tool_name.is_none() {
            data.tool_name = self.tool_name;
        }
        if data.summary.is_none() {
            data.summary = self.summary;
        }
        if data.permission_id.is_none() {
            data.permission_id = self.permission_id;
        }

        Ok(AgentEvent {
            agent,
            session_id: self.session_id,
            event,
            data,
        })
    }
}

fn parse_agent_type(value: &str) -> Option<AgentType> {
    match value {
        "claude-code" | "claude_code" | "claude" => Some(AgentType::ClaudeCode),
        "copilot" | "copilot-cli" | "copilot_cli" => Some(AgentType::Copilot),
        "codex" | "codex-cli" | "codex_cli" => Some(AgentType::Codex),
        _ => None,
    }
}

fn event_type_from_state(state: AgentState) -> EventType {
    match state {
        AgentState::Idle => EventType::Complete,
        AgentState::Thinking => EventType::Thinking,
        AgentState::ToolUse => EventType::ToolUse,
        AgentState::Error => EventType::Error,
        AgentState::WaitingPermission => EventType::PermissionRequest,
    }
}

fn parse_event_type(value: &str) -> Result<EventType, String> {
    let event = match value {
        "session_start" | "SessionStart" | "sessionStart" => EventType::SessionStart,
        "session_end" | "SessionEnd" | "sessionEnd" => EventType::SessionEnd,
        "thinking" | "UserPromptSubmit" | "userPromptSubmitted" => EventType::Thinking,
        "tool_use" | "PreToolUse" | "preToolUse" => EventType::ToolUse,
        "tool_result" | "PostToolUse" | "postToolUse" => EventType::ToolResult,
        "error" | "PostToolUseFailure" | "StopFailure" | "errorOccurred" => EventType::Error,
        "permission_request" | "PermissionRequest" => EventType::PermissionRequest,
        "complete" | "Stop" | "agentStop" | "Notification" | "notification" => EventType::Complete,
        "Elicitation" | "SubagentStart" | "subagentStart" | "SubagentStop" | "subagentStop"
        | "PreCompact" | "preCompact" | "PostCompact" | "WorktreeCreate" => EventType::ToolUse,
        other => return Err(format!("unsupported event: {other}")),
    };

    Ok(event)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    Pending,
    Approved,
    Denied,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StateChangePayload {
    pub state: AgentState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PermissionRequestPayload {
    pub permission_id: String,
    pub agent: AgentType,
    pub session_id: String,
    pub tool_name: String,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PermissionResolvedPayload {
    pub permission_id: String,
    pub status: PermissionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NavigatorStatus {
    pub current: StateChangePayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_permission: Option<PermissionRequestPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
}

#[derive(Clone, Debug)]
pub enum NavigatorEmission {
    StateChange(StateChangePayload),
    PermissionRequest(PermissionRequestPayload),
    PermissionResolved(PermissionResolvedPayload),
}
