use std::collections::HashMap;
use std::time::Instant;

use super::events::{
    AgentState, NavigatorSessionPayload, NavigatorSessionsPayload, StateChangePayload,
};
use super::state::SessionSnapshot;

pub fn derive_focus_snapshot(
    sessions: &HashMap<String, SessionSnapshot>,
    server_port: Option<u16>,
) -> StateChangePayload {
    if let Some(top) = sessions
        .values()
        .max_by_key(|session| (session.phase.priority(), session.updated_at))
    {
        StateChangePayload {
            state: top.phase.as_agent_state(),
            agent: Some(top.agent),
            session_id: Some(top.session_id.clone()),
            tool_name: top.tool_name.clone(),
            summary: top.summary.clone(),
            working_directory: top.working_directory.clone(),
            session_title: top.session_title.clone(),
            needs_attention: top.needs_attention,
            server_port,
            ai_talk_context: top.ai_talk_context.clone(),
        }
    } else {
        StateChangePayload {
            state: AgentState::Idle,
            agent: None,
            session_id: None,
            tool_name: None,
            summary: None,
            working_directory: None,
            session_title: None,
            needs_attention: None,
            server_port,
            ai_talk_context: None,
        }
    }
}

pub fn derive_sessions_payload(
    sessions: &HashMap<String, SessionSnapshot>,
    server_port: Option<u16>,
) -> NavigatorSessionsPayload {
    let mut items: Vec<&SessionSnapshot> = sessions.values().collect();
    items.sort_by_key(|session| {
        (
            std::cmp::Reverse(session.phase.priority()),
            std::cmp::Reverse(session.updated_at),
        )
    });

    NavigatorSessionsPayload {
        sessions: items.into_iter().map(into_payload).collect(),
        server_port,
    }
}

fn into_payload(session: &SessionSnapshot) -> NavigatorSessionPayload {
    NavigatorSessionPayload {
        agent: session.agent,
        session_id: session.session_id.clone(),
        phase: session.phase,
        state: session.phase.as_agent_state(),
        tool_name: session.tool_name.clone(),
        summary: session.summary.clone(),
        working_directory: session.working_directory.clone(),
        session_title: session.session_title.clone(),
        needs_attention: session.needs_attention,
        ai_talk_context: session.ai_talk_context.clone(),
    }
}

pub fn min_state_duration(state: AgentState) -> std::time::Duration {
    match state {
        AgentState::NeedsAttention => std::time::Duration::ZERO,
        AgentState::Error => std::time::Duration::from_secs(2),
        AgentState::Complete => std::time::Duration::from_secs(2),
        AgentState::ToolUse => std::time::Duration::from_secs(1),
        AgentState::Thinking => std::time::Duration::from_millis(1500),
        AgentState::Idle => std::time::Duration::ZERO,
    }
}

pub fn should_emit_focus(
    previous: Option<&StateChangePayload>,
    previous_at: Option<Instant>,
    next: &StateChangePayload,
    now: Instant,
) -> bool {
    let Some(previous) = previous else {
        return true;
    };

    if previous == next {
        return false;
    }

    if let Some(previous_at) = previous_at {
        let resumed_work_after_complete = matches!(previous.state, AgentState::Complete)
            && matches!(
                next.state,
                AgentState::Thinking
                    | AgentState::ToolUse
                    | AgentState::NeedsAttention
                    | AgentState::Error
            );

        if !resumed_work_after_complete
            && now.duration_since(previous_at) < min_state_duration(previous.state)
        {
            return false;
        }
    }

    true
}
