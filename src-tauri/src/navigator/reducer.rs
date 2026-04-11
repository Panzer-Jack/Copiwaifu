use std::time::Instant;

use super::events::{AgentEvent, EventType, SessionPhase};
use super::state::SessionSnapshot;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReduceResult {
    pub removed: bool,
}

pub fn reduce_session(session: &mut SessionSnapshot, event: &AgentEvent, now: Instant) -> ReduceResult {
    session.agent = event.agent;
    session.session_id = event.session_id.clone();
    session.updated_at = now;

    if let Some(working_directory) = &event.data.working_directory {
        session.working_directory = Some(working_directory.clone());
    }
    if let Some(session_title) = &event.data.session_title {
        session.session_title = Some(session_title.clone());
    }
    if let Some(summary) = &event.data.summary {
        session.summary = Some(summary.clone());
    }

    match event.event {
        EventType::SessionStart => {
            session.phase = SessionPhase::Idle;
            session.tool_name = None;
            session.needs_attention = Some(false);
            session.summary = event.data.summary.clone();
            session.started_at = now;
        }
        EventType::SessionEnd => {
            return ReduceResult { removed: true };
        }
        EventType::Thinking => {
            session.phase = SessionPhase::Processing;
            session.tool_name = None;
            session.needs_attention = Some(false);
        }
        EventType::ToolUse => {
            session.phase = SessionPhase::RunningTool;
            session.tool_name = event
                .data
                .tool_name
                .clone()
                .or_else(|| session.tool_name.clone());
            session.needs_attention = Some(false);
        }
        EventType::ToolResult => {
            session.phase = SessionPhase::Processing;
            session.tool_name = None;
            session.needs_attention = Some(false);
        }
        EventType::Error => {
            session.phase = SessionPhase::Error;
            session.tool_name = event
                .data
                .tool_name
                .clone()
                .or_else(|| session.tool_name.clone());
            session.needs_attention = Some(false);
        }
        EventType::Complete => {
            session.phase = SessionPhase::Completed;
            session.tool_name = None;
            session.needs_attention = Some(false);
        }
        EventType::NeedsAttention => {
            session.phase = SessionPhase::WaitingAttention;
            session.tool_name = event
                .data
                .tool_name
                .clone()
                .or_else(|| session.tool_name.clone());
            session.needs_attention = Some(true);
        }
    }

    if event.data.needs_attention == Some(false) && event.event != EventType::NeedsAttention {
        session.needs_attention = Some(false);
    }

    ReduceResult { removed: false }
}
