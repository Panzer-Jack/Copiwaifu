use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::{
    agent::{session_key, SESSION_TTL},
    events::{
        AgentEvent, AgentState, AgentType, AiTalkContext, AiTalkEventDigest, EventType,
        NavigatorEmission, NavigatorSessionsPayload, NavigatorStatus, SessionPhase,
        StateChangePayload,
    },
    presentation, reducer, session_store,
};

#[derive(Clone, Debug)]
pub struct SessionSnapshot {
    pub agent: AgentType,
    pub session_id: String,
    pub phase: SessionPhase,
    pub last_event_type: Option<EventType>,
    pub tool_name: Option<String>,
    pub summary: Option<String>,
    pub working_directory: Option<String>,
    pub session_title: Option<String>,
    pub needs_attention: Option<bool>,
    pub events: Vec<AiTalkEventDigest>,
    pub last_meaningful_summary: Option<String>,
    pub ai_talk_context: Option<AiTalkContext>,
    pub turn_index: u64,
    pub turn_fingerprint: Option<String>,
    pub terminal_state: Option<AgentState>,
    pub started_at: Instant,
    pub updated_at: Instant,
    pub started_at_ms: u64,
    pub updated_at_ms: u64,
}

impl SessionSnapshot {
    fn new(agent: AgentType, session_id: &str, now: Instant, now_ms: u64) -> Self {
        Self {
            agent,
            session_id: session_id.to_string(),
            phase: SessionPhase::Idle,
            last_event_type: None,
            tool_name: None,
            summary: None,
            working_directory: None,
            session_title: None,
            needs_attention: Some(false),
            events: Vec::new(),
            last_meaningful_summary: None,
            ai_talk_context: None,
            turn_index: 0,
            turn_fingerprint: None,
            terminal_state: None,
            started_at: now,
            updated_at: now,
            started_at_ms: now_ms,
            updated_at_ms: now_ms,
        }
    }
}

pub struct NavigatorState {
    sessions: HashMap<String, SessionSnapshot>,
    server_port: Option<u16>,
    last_focus_snapshot: Option<StateChangePayload>,
    last_focus_at: Option<Instant>,
    last_sessions_snapshot: Option<NavigatorSessionsPayload>,
    ai_talk_claims: HashMap<String, Instant>,
    ai_talk_context_cache: HashMap<String, (AiTalkContext, Instant)>,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            server_port: None,
            last_focus_snapshot: None,
            last_focus_at: None,
            last_sessions_snapshot: None,
            ai_talk_claims: HashMap::new(),
            ai_talk_context_cache: HashMap::new(),
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
        let now_ms = current_time_ms();
        let key = session_key(&event.agent, &event.session_id);
        let mut persist_snapshot = None;
        let mut ended_session = None;
        let mut cached_context = None;

        let removed = {
            let session = self.sessions.entry(key.clone()).or_insert_with(|| {
                SessionSnapshot::new(event.agent, &event.session_id, now, now_ms)
            });
            let reduced = reducer::reduce_session(session, &event, now, now_ms);
            if reduced.removed {
                cached_context = session.ai_talk_context.clone();
                ended_session = Some((event.agent, event.session_id.clone(), now_ms));
            } else {
                persist_snapshot = Some(session.clone());
            }
            reduced.removed
        };

        if removed {
            self.sessions.remove(&key);
        }

        if let Some(context) = cached_context {
            self.ai_talk_context_cache
                .insert(key.clone(), (context, now));
        }

        if let Some(snapshot) = persist_snapshot {
            if let Err(err) = session_store::persist_snapshot(&snapshot) {
                eprintln!("[session_store] persist failed: {err}");
            }
        }

        if let Some((agent, session_id, ended_at_ms)) = ended_session {
            if let Err(err) = session_store::mark_session_ended(agent, &session_id, ended_at_ms) {
                eprintln!("[session_store] mark ended failed: {err}");
            }
        }

        self.collect_emissions(now)
    }

    pub fn claim_ai_talk_context(
        &mut self,
        agent: AgentType,
        session_id: &str,
        state: AgentState,
    ) -> Option<AiTalkContext> {
        if !matches!(state, AgentState::Complete | AgentState::Error) {
            return None;
        }

        let key = session_key(&agent, session_id);
        let context = self
            .sessions
            .get(&key)
            .and_then(|session| session.ai_talk_context.clone())
            .or_else(|| {
                self.ai_talk_context_cache
                    .get(&key)
                    .map(|(context, _)| context.clone())
            })?;
        if context.state != state || !context.has_context {
            return None;
        }

        let claim_key = format!(
            "{}::{session_id}::{:?}::{}",
            agent.as_str(),
            state,
            context.turn_index
        );
        if self.ai_talk_claims.contains_key(&claim_key) {
            return None;
        }

        self.ai_talk_claims.insert(claim_key, Instant::now());
        Some(context)
    }

    pub fn cleanup_stale(&mut self) -> Vec<NavigatorEmission> {
        self.cleanup_stale_at(Instant::now())
    }

    fn cleanup_stale_at(&mut self, now: Instant) -> Vec<NavigatorEmission> {
        self.sessions
            .retain(|_, session| now.duration_since(session.updated_at) < SESSION_TTL);

        self.ai_talk_claims
            .retain(|_, claimed_at| now.duration_since(*claimed_at) < Duration::from_secs(300));
        self.ai_talk_context_cache
            .retain(|_, (_, cached_at)| now.duration_since(*cached_at) < Duration::from_secs(300));

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

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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

    fn turn_start_event(agent: AgentType, session_id: &str, summary: &str) -> AgentEvent {
        AgentEvent {
            agent,
            session_id: session_id.to_string(),
            event: EventType::Thinking,
            data: EventData {
                summary: Some(summary.to_string()),
                turn_start: true,
                turn_fingerprint: Some(summary.to_string()),
                ..EventData::default()
            },
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

        let second = state.apply_event(turn_start_event(
            AgentType::Codex,
            "same-session",
            "follow-up",
        ));
        assert_eq!(state_from(second), AgentState::Thinking);
    }

    #[test]
    fn stale_thinking_snapshot_after_complete_does_not_resume_turn() {
        let mut state = NavigatorState::new();

        let first = state.apply_event(event(AgentType::Codex, "same-session", EventType::Complete));
        assert_eq!(state_from(first), AgentState::Complete);

        let second =
            state.apply_event(event(AgentType::Codex, "same-session", EventType::Thinking));
        assert!(!has_state_change(&second));
    }

    #[test]
    fn ai_talk_claims_once_per_terminal_turn() {
        let mut state = NavigatorState::new();

        state.apply_event(turn_start_event(AgentType::Codex, "same-session", "first"));
        state.apply_event(event(AgentType::Codex, "same-session", EventType::Complete));

        assert!(state
            .claim_ai_talk_context(AgentType::Codex, "same-session", AgentState::Complete)
            .is_some());
        assert!(state
            .claim_ai_talk_context(AgentType::Codex, "same-session", AgentState::Complete)
            .is_none());

        state.apply_event(turn_start_event(AgentType::Codex, "same-session", "second"));
        state.apply_event(event(AgentType::Codex, "same-session", EventType::Complete));

        assert!(state
            .claim_ai_talk_context(AgentType::Codex, "same-session", AgentState::Complete)
            .is_some());
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
