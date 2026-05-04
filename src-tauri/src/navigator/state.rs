use std::{collections::HashMap, time::Instant};

use super::{
    agent::{session_key, SESSION_TTL},
    events::{
        AgentEvent, AgentType, NavigatorEmission, NavigatorSessionsPayload, NavigatorStatus,
        SessionPhase, StateChangePayload,
    },
    presentation, reducer,
};

#[derive(Clone, Debug)]
pub struct SessionSnapshot {
    pub agent: AgentType,
    pub session_id: String,
    pub phase: SessionPhase,
    pub tool_name: Option<String>,
    pub summary: Option<String>,
    pub working_directory: Option<String>,
    pub session_title: Option<String>,
    pub needs_attention: Option<bool>,
    pub started_at: Instant,
    pub updated_at: Instant,
}

impl SessionSnapshot {
    fn new(agent: AgentType, session_id: &str, now: Instant) -> Self {
        Self {
            agent,
            session_id: session_id.to_string(),
            phase: SessionPhase::Idle,
            tool_name: None,
            summary: None,
            working_directory: None,
            session_title: None,
            needs_attention: Some(false),
            started_at: now,
            updated_at: now,
        }
    }
}

pub struct NavigatorState {
    sessions: HashMap<String, SessionSnapshot>,
    server_port: Option<u16>,
    last_focus_snapshot: Option<StateChangePayload>,
    last_focus_at: Option<Instant>,
    last_sessions_snapshot: Option<NavigatorSessionsPayload>,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            server_port: None,
            last_focus_snapshot: None,
            last_focus_at: None,
            last_sessions_snapshot: None,
        }
    }

    pub fn set_server_port(&mut self, port: u16) {
        self.server_port = Some(port);
    }

    pub fn snapshot(&self) -> NavigatorStatus {
        NavigatorStatus {
            current: self.last_focus_snapshot.clone().unwrap_or_else(|| {
                presentation::derive_focus_snapshot(&self.sessions, self.server_port)
            }),
            server_port: self.server_port,
        }
    }

    pub fn sessions_snapshot(&self) -> NavigatorSessionsPayload {
        self.last_sessions_snapshot.clone().unwrap_or_else(|| {
            presentation::derive_sessions_payload(&self.sessions, self.server_port)
        })
    }

    pub fn apply_event(&mut self, event: AgentEvent) -> Vec<NavigatorEmission> {
        let now = Instant::now();
        let key = session_key(&event.agent, &event.session_id);

        let removed = {
            let session = self
                .sessions
                .entry(key.clone())
                .or_insert_with(|| SessionSnapshot::new(event.agent, &event.session_id, now));
            reducer::reduce_session(session, &event, now).removed
        };

        if removed {
            self.sessions.remove(&key);
        }

        self.collect_emissions(now)
    }

    pub fn cleanup_stale(&mut self) -> Vec<NavigatorEmission> {
        self.cleanup_stale_at(Instant::now())
    }

    fn cleanup_stale_at(&mut self, now: Instant) -> Vec<NavigatorEmission> {
        self.sessions
            .retain(|_, session| now.duration_since(session.updated_at) < SESSION_TTL);

        self.collect_emissions(now)
    }

    fn collect_emissions(&mut self, now: Instant) -> Vec<NavigatorEmission> {
        let mut emissions = Vec::new();

        let focus = presentation::derive_focus_snapshot(&self.sessions, self.server_port);
        if presentation::should_emit_focus(
            self.last_focus_snapshot.as_ref(),
            self.last_focus_at,
            &focus,
            now,
        ) {
            self.last_focus_snapshot = Some(focus.clone());
            self.last_focus_at = Some(now);
            emissions.push(NavigatorEmission::StateChange(focus));
        }

        let sessions = presentation::derive_sessions_payload(&self.sessions, self.server_port);
        if self.last_sessions_snapshot.as_ref() != Some(&sessions) {
            self.last_sessions_snapshot = Some(sessions.clone());
            emissions.push(NavigatorEmission::SessionsChanged(sessions));
        }

        emissions
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::events::{AgentEvent, AgentState, AgentType, EventData, EventType};
    use super::{NavigatorEmission, NavigatorState};

    fn state_from(emissions: Vec<NavigatorEmission>) -> AgentState {
        match emissions.iter().find_map(|emission| {
            if let NavigatorEmission::StateChange(payload) = emission {
                Some(payload.state)
            } else {
                None
            }
        }) {
            Some(state) => state,
            None => panic!("expected a state change emission"),
        }
    }

    fn event(agent: AgentType, session_id: &str, event: EventType) -> AgentEvent {
        AgentEvent {
            agent,
            session_id: session_id.to_string(),
            event,
            data: EventData::default(),
        }
    }

    fn has_state_change(emissions: &[NavigatorEmission]) -> bool {
        emissions
            .iter()
            .any(|emission| matches!(emission, NavigatorEmission::StateChange(_)))
    }

    #[test]
    fn active_session_beats_stale_complete_snapshot() {
        let mut state = NavigatorState::new();

        let first = state.apply_event(event(AgentType::Codex, "done-session", EventType::Complete));
        assert_eq!(state_from(first), AgentState::Complete);

        let second = state.apply_event(event(
            AgentType::Codex,
            "active-session",
            EventType::Thinking,
        ));
        assert_eq!(state_from(second), AgentState::Thinking);
    }

    #[test]
    fn resumed_work_is_not_blocked_by_complete_min_duration() {
        let mut state = NavigatorState::new();

        let first = state.apply_event(event(AgentType::Codex, "same-session", EventType::Complete));
        assert_eq!(state_from(first), AgentState::Complete);

        let second =
            state.apply_event(event(AgentType::Codex, "same-session", EventType::Thinking));
        assert_eq!(state_from(second), AgentState::Thinking);
    }

    #[test]
    fn cleanup_tick_releases_delayed_idle_after_complete_min_duration() {
        let mut state = NavigatorState::new();

        let first = state.apply_event(event(AgentType::Codex, "same-session", EventType::Complete));
        assert_eq!(state_from(first), AgentState::Complete);

        let second = state.apply_event(event(
            AgentType::Codex,
            "same-session",
            EventType::SessionEnd,
        ));
        assert!(!has_state_change(&second));

        let delayed_at = state
            .last_focus_at
            .expect("complete should set focus timestamp")
            + Duration::from_secs(3);
        let delayed = state.cleanup_stale_at(delayed_at);

        assert_eq!(state_from(delayed), AgentState::Idle);
    }
}
