use serde::{Deserialize, Serialize};

use super::providers;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    ClaudeCode,
    Copilot,
    Codex,
    Gemini,
    OpenCode,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Copilot => "copilot",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
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
    Complete,
    NeedsAttention,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Thinking,
    ToolUse,
    Error,
    Complete,
    NeedsAttention,
}

impl AgentState {
    pub fn priority(&self) -> u8 {
        match self {
            Self::NeedsAttention => 5,
            Self::Error => 4,
            Self::ToolUse => 3,
            Self::Thinking => 2,
            Self::Complete => 1,
            Self::Idle => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Idle,
    Processing,
    RunningTool,
    WaitingAttention,
    Completed,
    Error,
}

impl SessionPhase {
    pub fn as_agent_state(&self) -> AgentState {
        match self {
            Self::Idle => AgentState::Idle,
            Self::Processing => AgentState::Thinking,
            Self::RunningTool => AgentState::ToolUse,
            Self::WaitingAttention => AgentState::NeedsAttention,
            Self::Completed => AgentState::Complete,
            Self::Error => AgentState::Error,
        }
    }

    pub fn priority(&self) -> u8 {
        self.as_agent_state().priority()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventData {
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub session_title: Option<String>,
    #[serde(default)]
    pub needs_attention: Option<bool>,
    #[serde(default)]
    pub turn_start: bool,
    #[serde(default)]
    pub turn_fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTalkEventDigest {
    pub event_type: EventType,
    pub timestamp_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub informative: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTalkContext {
    pub agent: AgentType,
    pub session_id: String,
    pub state: AgentState,
    pub phase: SessionPhase,
    pub turn_index: u64,
    pub updated_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_event_type: Option<EventType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_meaningful_summary: Option<String>,
    pub has_context: bool,
    pub missing_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<AiTalkEventDigest>,
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
    pub working_directory: Option<String>,
    #[serde(default)]
    pub session_title: Option<String>,
    #[serde(default)]
    pub needs_attention: Option<bool>,
    #[serde(default)]
    pub turn_start: Option<bool>,
    #[serde(default)]
    pub turn_fingerprint: Option<String>,
}

impl IncomingHookEvent {
    pub fn into_agent_event(self) -> Result<AgentEvent, String> {
        let agent = self
            .agent
            .or_else(|| {
                self.agent_id
                    .as_deref()
                    .and_then(providers::parse_agent_type)
            })
            .ok_or_else(|| "missing agent".to_string())?;

        let event = if let Some(raw_event) = self.event.as_deref() {
            providers::normalize_event(agent, raw_event)?
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
        if data.working_directory.is_none() {
            data.working_directory = self.working_directory;
        }
        if data.session_title.is_none() {
            data.session_title = self.session_title;
        }
        if data.needs_attention.is_none() {
            data.needs_attention = self.needs_attention;
        }
        if !data.turn_start {
            data.turn_start = self.turn_start.unwrap_or(false);
        }
        if data.turn_fingerprint.is_none() {
            data.turn_fingerprint = self.turn_fingerprint;
        }

        Ok(AgentEvent {
            agent,
            session_id: self.session_id,
            event,
            data,
        })
    }
}

fn event_type_from_state(state: AgentState) -> EventType {
    match state {
        AgentState::Idle => EventType::Complete,
        AgentState::Thinking => EventType::Thinking,
        AgentState::ToolUse => EventType::ToolUse,
        AgentState::Error => EventType::Error,
        AgentState::Complete => EventType::Complete,
        AgentState::NeedsAttention => EventType::NeedsAttention,
    }
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
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_attention: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_talk_context: Option<AiTalkContext>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NavigatorSessionPayload {
    pub agent: AgentType,
    pub session_id: String,
    pub phase: SessionPhase,
    pub state: AgentState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_attention: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_talk_context: Option<AiTalkContext>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NavigatorSessionsPayload {
    pub sessions: Vec<NavigatorSessionPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NavigatorStatus {
    pub current: StateChangePayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
}

#[derive(Clone, Debug)]
pub enum NavigatorEmission {
    StateChange(StateChangePayload),
    SessionsChanged(NavigatorSessionsPayload),
}
