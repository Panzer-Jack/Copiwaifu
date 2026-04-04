use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use super::{
    agent::{session_key, SESSION_TTL},
    events::{
        AgentEvent, AgentState, EventType, NavigatorEmission, NavigatorStatus, StateChangePayload,
    },
};

#[derive(Clone, Debug)]
struct SessionRecord {
    agent: super::events::AgentType,
    session_id: String,
    state: AgentState,
    tool_name: Option<String>,
    summary: Option<String>,
    updated_at: Instant,
}

pub struct NavigatorState {
    sessions: HashMap<String, SessionRecord>,
    server_port: Option<u16>,
    last_snapshot: Option<StateChangePayload>,
    last_snapshot_at: Option<Instant>,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            server_port: None,
            last_snapshot: None,
            last_snapshot_at: None,
        }
    }

    pub fn set_server_port(&mut self, port: u16) {
        self.server_port = Some(port);
    }

    pub fn snapshot(&self) -> NavigatorStatus {
        NavigatorStatus {
            current: self
                .last_snapshot
                .clone()
                .unwrap_or_else(|| self.compute_raw_snapshot()),
            server_port: self.server_port,
        }
    }

    pub fn apply_event(&mut self, event: AgentEvent) -> Vec<NavigatorEmission> {
        let now = Instant::now();

        match event.event {
            EventType::SessionStart => self.upsert_session(
                event.agent,
                &event.session_id,
                AgentState::Idle,
                None,
                None,
                now,
            ),
            EventType::SessionEnd => {
                let key = session_key(&event.agent, &event.session_id);
                self.sessions.remove(&key);
            }
            EventType::Thinking => self.upsert_session(
                event.agent,
                &event.session_id,
                AgentState::Thinking,
                event.data.tool_name.clone(),
                event.data.summary.clone(),
                now,
            ),
            EventType::ToolUse => self.upsert_session(
                event.agent,
                &event.session_id,
                AgentState::ToolUse,
                event.data.tool_name.clone(),
                event.data.summary.clone(),
                now,
            ),
            EventType::ToolResult => self.upsert_session(
                event.agent,
                &event.session_id,
                AgentState::Thinking,
                event.data.tool_name.clone(),
                event.data.summary.clone(),
                now,
            ),
            EventType::Error => self.upsert_session(
                event.agent,
                &event.session_id,
                AgentState::Error,
                event.data.tool_name.clone(),
                event.data.summary.clone(),
                now,
            ),
            EventType::Complete => self.upsert_session(
                event.agent,
                &event.session_id,
                AgentState::Idle,
                None,
                event.data.summary.clone(),
                now,
            ),
        }

        self.maybe_emit_state_change()
    }

    pub fn cleanup_stale(&mut self) -> Vec<NavigatorEmission> {
        let now = Instant::now();
        let before = self.sessions.len();

        self.sessions
            .retain(|_, session| now.duration_since(session.updated_at) < SESSION_TTL);

        if self.sessions.len() != before {
            self.maybe_emit_state_change()
        } else {
            vec![]
        }
    }

    fn upsert_session(
        &mut self,
        agent: super::events::AgentType,
        session_id: &str,
        state: AgentState,
        tool_name: Option<String>,
        summary: Option<String>,
        now: Instant,
    ) {
        let key = session_key(&agent, session_id);
        self.sessions.insert(
            key,
            SessionRecord {
                agent,
                session_id: session_id.to_string(),
                state,
                tool_name,
                summary,
                updated_at: now,
            },
        );
    }

    fn maybe_emit_state_change(&mut self) -> Vec<NavigatorEmission> {
        let raw = self.compute_raw_snapshot();
        let now = Instant::now();

        if let Some(last) = &self.last_snapshot {
            if *last == raw {
                return vec![];
            }

            if let Some(last_at) = self.last_snapshot_at {
                let min_dur = min_state_duration(last.state);
                if now.duration_since(last_at) < min_dur {
                    return vec![];
                }
            }
        }

        self.last_snapshot = Some(raw.clone());
        self.last_snapshot_at = Some(now);
        vec![NavigatorEmission::StateChange(raw)]
    }

    fn compute_raw_snapshot(&self) -> StateChangePayload {
        if let Some(top) = self
            .sessions
            .values()
            .max_by_key(|s| (s.state.priority(), s.updated_at))
        {
            StateChangePayload {
                state: top.state,
                agent: Some(top.agent),
                session_id: Some(top.session_id.clone()),
                tool_name: top.tool_name.clone(),
                summary: top.summary.clone(),
                server_port: self.server_port,
            }
        } else {
            StateChangePayload {
                state: AgentState::Idle,
                agent: None,
                session_id: None,
                tool_name: None,
                summary: None,
                server_port: self.server_port,
            }
        }
    }
}

fn min_state_duration(state: AgentState) -> Duration {
    match state {
        AgentState::Error => Duration::from_secs(2),
        AgentState::ToolUse => Duration::from_secs(1),
        AgentState::Thinking => Duration::from_millis(1500),
        AgentState::Idle => Duration::ZERO,
    }
}
