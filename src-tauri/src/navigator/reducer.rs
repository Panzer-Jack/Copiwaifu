use std::time::Instant;

use super::events::{
    AgentEvent, AgentState, AgentType, AiTalkContext, AiTalkEventDigest, EventType, SessionPhase,
};
use super::state::SessionSnapshot;

const MAX_EVENT_HISTORY: usize = 20;
const MAX_AI_TALK_EVENTS: usize = 20;
const MAX_LOW_PRIORITY_EVENTS: usize = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReduceResult {
    pub removed: bool,
}

pub fn reduce_session(
    session: &mut SessionSnapshot,
    event: &AgentEvent,
    now: Instant,
    now_ms: u64,
) -> ReduceResult {
    session.agent = event.agent;
    session.session_id = event.session_id.clone();
    session.updated_at = now;
    session.updated_at_ms = now_ms;

    if event.event == EventType::SessionStart {
        session.phase = SessionPhase::Idle;
        session.last_event_type = None;
        session.tool_name = None;
        session.summary = None;
        session.working_directory = None;
        session.session_title = None;
        session.needs_attention = Some(false);
        session.events.clear();
        session.last_meaningful_summary = None;
        session.ai_talk_context = None;
        session.turn_index = 0;
        session.turn_fingerprint = None;
        session.terminal_state = None;
        session.started_at = now;
        session.started_at_ms = now_ms;
    }

    if starts_new_turn(session, event) {
        session.turn_index += 1;
        session.turn_fingerprint = turn_fingerprint(event);
        session.terminal_state = None;
    }

    let terminal_state = terminal_state_for_event(event.event);
    let stale_after_terminal = session.terminal_state.is_some()
        && !event.data.turn_start
        && terminal_state.is_none()
        && event.event != EventType::SessionEnd;

    push_event_digest(session, event, now_ms);

    if !stale_after_terminal {
        if let Some(working_directory) = &event.data.working_directory {
            session.working_directory = Some(working_directory.clone());
        }
        if let Some(session_title) = &event.data.session_title {
            session.session_title = Some(session_title.clone());
        }
        if let Some(summary) = &event.data.summary {
            session.summary = Some(summary.clone());
        }
    }

    let duplicate_terminal =
        terminal_state.is_some_and(|state| session.terminal_state == Some(state));

    match event.event {
        EventType::SessionStart => {
            session.phase = SessionPhase::Idle;
            session.tool_name = None;
            session.needs_attention = Some(false);
            session.summary = event.data.summary.clone();
        }
        EventType::SessionEnd => {
            return ReduceResult { removed: true };
        }
        EventType::Thinking => {
            if !stale_after_terminal {
                session.phase = SessionPhase::Processing;
                session.tool_name = None;
                session.needs_attention = Some(false);
            }
        }
        EventType::ToolUse => {
            if !stale_after_terminal {
                session.phase = SessionPhase::RunningTool;
                session.tool_name = event
                    .data
                    .tool_name
                    .clone()
                    .or_else(|| session.tool_name.clone());
                session.needs_attention = Some(false);
            }
        }
        EventType::ToolResult => {
            if !stale_after_terminal {
                session.phase = SessionPhase::Processing;
                session.tool_name = None;
                session.needs_attention = Some(false);
            }
        }
        EventType::Error => {
            session.phase = SessionPhase::Error;
            session.terminal_state = Some(AgentState::Error);
            session.tool_name = event
                .data
                .tool_name
                .clone()
                .or_else(|| session.tool_name.clone());
            session.needs_attention = Some(false);
        }
        EventType::Complete => {
            session.phase = SessionPhase::Completed;
            session.terminal_state = Some(AgentState::Complete);
            session.tool_name = None;
            session.needs_attention = Some(false);
        }
        EventType::NeedsAttention => {
            if !stale_after_terminal {
                session.phase = SessionPhase::WaitingAttention;
                session.tool_name = event
                    .data
                    .tool_name
                    .clone()
                    .or_else(|| session.tool_name.clone());
                session.needs_attention = Some(true);
            }
        }
    }

    if !stale_after_terminal
        && event.data.needs_attention == Some(false)
        && event.event != EventType::NeedsAttention
    {
        session.needs_attention = Some(false);
    }

    if !stale_after_terminal {
        session.last_event_type = Some(event.event);
        session.last_meaningful_summary = best_session_summary(session);
    }
    if !duplicate_terminal && !stale_after_terminal {
        session.ai_talk_context = Some(build_ai_talk_context(session, event.event, now_ms));
    }

    ReduceResult { removed: false }
}

fn starts_new_turn(session: &SessionSnapshot, event: &AgentEvent) -> bool {
    if !event.data.turn_start {
        return false;
    }

    if session.terminal_state.is_some() || session.turn_index == 0 {
        return true;
    }

    match turn_fingerprint(event) {
        Some(fingerprint) => session.turn_fingerprint.as_ref() != Some(&fingerprint),
        None => false,
    }
}

fn turn_fingerprint(event: &AgentEvent) -> Option<String> {
    event
        .data
        .turn_fingerprint
        .as_deref()
        .or(event.data.session_title.as_deref())
        .or(event.data.summary.as_deref())
        .map(clean_summary)
        .filter(|value| !value.is_empty())
}

fn terminal_state_for_event(event_type: EventType) -> Option<AgentState> {
    match event_type {
        EventType::Complete => Some(AgentState::Complete),
        EventType::Error => Some(AgentState::Error),
        _ => None,
    }
}

fn push_event_digest(session: &mut SessionSnapshot, event: &AgentEvent, timestamp_ms: u64) {
    let summary = event
        .data
        .summary
        .as_deref()
        .map(clean_summary)
        .filter(|value| !value.is_empty());
    let tool_name = event
        .data
        .tool_name
        .as_deref()
        .map(clean_summary)
        .filter(|value| !value.is_empty());
    let informative = is_meaningful_summary(
        summary.as_deref(),
        tool_name.as_deref(),
        event.agent,
        event.event,
    );

    session.events.push(AiTalkEventDigest {
        event_type: event.event,
        timestamp_ms,
        tool_name,
        summary,
        informative,
    });

    if session.events.len() > MAX_EVENT_HISTORY {
        let overflow = session.events.len() - MAX_EVENT_HISTORY;
        session.events.drain(0..overflow);
    }
}

fn build_ai_talk_context(
    session: &SessionSnapshot,
    recent_event_type: EventType,
    updated_at_ms: u64,
) -> AiTalkContext {
    let recent_tool_name = session.tool_name.clone().or_else(|| {
        session
            .events
            .iter()
            .rev()
            .find_map(|event| event.tool_name.clone())
    });
    let recent_summary = best_session_summary(session)
        .or_else(|| session.last_meaningful_summary.clone())
        .or_else(|| {
            session
                .summary
                .as_deref()
                .map(clean_summary)
                .filter(|value| {
                    is_meaningful_summary(
                        Some(value.as_str()),
                        recent_tool_name.as_deref(),
                        session.agent,
                        recent_event_type,
                    )
                })
        });
    let has_context = recent_summary.is_some() || session.session_title.is_some();
    let mut missing_fields = Vec::new();

    if session.working_directory.is_none() {
        missing_fields.push("workingDirectory".to_string());
    }
    if session.session_title.is_none() {
        missing_fields.push("sessionTitle".to_string());
    }
    if recent_summary.is_none() {
        missing_fields.push("lastMeaningfulSummary".to_string());
    }
    if recent_tool_name.is_none() {
        missing_fields.push("toolName".to_string());
    }

    AiTalkContext {
        agent: session.agent,
        session_id: session.session_id.clone(),
        state: session.phase.as_agent_state(),
        phase: session.phase,
        turn_index: session.turn_index,
        updated_at_ms,
        working_directory: session.working_directory.clone(),
        session_title: session.session_title.clone(),
        tool_name: recent_tool_name,
        recent_event_type: Some(recent_event_type),
        recent_summary: recent_summary.clone(),
        last_meaningful_summary: recent_summary,
        has_context,
        missing_fields,
        events: prioritized_events(&session.events),
    }
}

pub(crate) fn is_meaningful_summary(
    summary: Option<&str>,
    tool_name: Option<&str>,
    agent: AgentType,
    event_type: EventType,
) -> bool {
    let Some(summary) = summary.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };

    let normalized = normalize_summary(summary);
    if normalized.is_empty() {
        return false;
    }

    let agent_name = normalize_summary(agent.as_str());
    let event_name = normalize_summary(&format!("{event_type:?}"));
    if normalized == agent_name || normalized == event_name {
        return false;
    }

    if let Some(tool_name) = tool_name {
        if normalized == normalize_summary(tool_name) {
            return false;
        }
    }

    if matches!(
        normalized.as_str(),
        "idle"
            | "working"
            | "complete"
            | "completed"
            | "error"
            | "thinking"
            | "tooluse"
            | "toolresult"
            | "sessionstart"
            | "sessionend"
    ) {
        return false;
    }

    let lower = summary.trim().to_lowercase();
    if lower.starts_with("waiting ") || lower.starts_with("waiting for ") {
        return false;
    }
    if summary.trim().starts_with('等') && summary.contains("操作") {
        return false;
    }
    if lower.starts_with("running ") || lower.starts_with("finished ") {
        return false;
    }
    if lower.ends_with(" session started")
        || lower.ends_with(" session closed")
        || lower.ends_with(" session archived")
        || lower.ends_with(" finished this turn")
    {
        return false;
    }

    true
}

#[cfg(test)]
fn best_meaningful_summary(events: &[AiTalkEventDigest]) -> Option<String> {
    best_meaningful_event(events).map(|(_, _, summary)| summary.clone())
}

fn best_session_summary(session: &SessionSnapshot) -> Option<String> {
    let best_event = best_meaningful_event(&session.events);
    if best_event.is_some_and(|(priority, _, _)| priority >= 4) {
        return best_event.map(|(_, _, summary)| summary.clone());
    }

    if let Some(session_title) =
        session
            .session_title
            .as_deref()
            .map(clean_summary)
            .filter(|value| {
                is_meaningful_summary(
                    Some(value.as_str()),
                    session.tool_name.as_deref(),
                    session.agent,
                    EventType::Thinking,
                )
            })
    {
        return Some(session_title);
    }

    best_event.map(|(_, _, summary)| summary.clone())
}

fn best_meaningful_event(events: &[AiTalkEventDigest]) -> Option<(u8, u64, &String)> {
    events
        .iter()
        .filter(|event| event.informative)
        .filter_map(|event| {
            event
                .summary
                .as_ref()
                .map(|summary| (summary_priority(event), event.timestamp_ms, summary))
        })
        .max_by_key(|(priority, timestamp_ms, _)| (*priority, *timestamp_ms))
}

fn summary_priority(event: &AiTalkEventDigest) -> u8 {
    match event.event_type {
        EventType::Complete | EventType::Error => 5,
        EventType::Thinking => 4,
        EventType::NeedsAttention => 3,
        EventType::ToolResult => 2,
        EventType::ToolUse => 1,
        EventType::SessionStart | EventType::SessionEnd => 0,
    }
}

fn is_high_priority_event(event_type: EventType) -> bool {
    matches!(
        event_type,
        EventType::Thinking | EventType::Complete | EventType::Error | EventType::NeedsAttention
    )
}

fn prioritized_events(events: &[AiTalkEventDigest]) -> Vec<AiTalkEventDigest> {
    let high: Vec<_> = events
        .iter()
        .filter(|e| e.informative && is_high_priority_event(e.event_type))
        .cloned()
        .collect();
    let low: Vec<_> = events
        .iter()
        .rev()
        .filter(|e| e.informative && !is_high_priority_event(e.event_type))
        .take(MAX_LOW_PRIORITY_EVENTS)
        .cloned()
        .collect();
    let mut result: Vec<_> = high.into_iter().chain(low).collect();
    result.sort_by_key(|e| e.timestamp_ms);
    result.truncate(MAX_AI_TALK_EVENTS);
    result
}

fn clean_summary(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_summary(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::{
        best_meaningful_summary, best_session_summary, prioritized_events, AgentType,
        AiTalkEventDigest, EventType, SessionPhase, SessionSnapshot,
    };

    fn digest(event_type: EventType, timestamp_ms: u64, summary: &str) -> AiTalkEventDigest {
        AiTalkEventDigest {
            event_type,
            timestamp_ms,
            tool_name: None,
            summary: Some(summary.to_string()),
            informative: true,
        }
    }

    #[test]
    fn user_intent_beats_later_tool_command_for_ai_talk() {
        let events = vec![
            digest(EventType::Thinking, 1, "帮我优化 Claude 逻辑"),
            digest(
                EventType::ToolUse,
                2,
                "git diff -- src/windows/MainWindow.vue",
            ),
            digest(
                EventType::ToolResult,
                3,
                "git diff -- src/windows/MainWindow.vue",
            ),
        ];

        assert_eq!(
            best_meaningful_summary(&events).as_deref(),
            Some("帮我优化 Claude 逻辑")
        );
    }

    #[test]
    fn completion_result_beats_previous_user_intent() {
        let events = vec![
            digest(EventType::Thinking, 1, "帮我优化 Claude 逻辑"),
            digest(
                EventType::Complete,
                2,
                "Claude summary extraction has been updated",
            ),
        ];

        assert_eq!(
            best_meaningful_summary(&events).as_deref(),
            Some("Claude summary extraction has been updated")
        );
    }

    #[test]
    fn session_title_beats_tool_command_when_user_intent_event_was_trimmed() {
        let now = Instant::now();
        let session = SessionSnapshot {
            agent: AgentType::ClaudeCode,
            session_id: "claude-session".to_string(),
            phase: SessionPhase::Processing,
            last_event_type: Some(EventType::ToolResult),
            tool_name: Some("Read".to_string()),
            summary: Some("/tmp/project/src-tauri/src/ai_talk.rs".to_string()),
            working_directory: Some("/tmp/project".to_string()),
            session_title: Some("详细细化一下".to_string()),
            needs_attention: Some(false),
            events: vec![
                digest(
                    EventType::ToolUse,
                    2,
                    "/tmp/project/src-tauri/src/ai_talk.rs",
                ),
                digest(
                    EventType::ToolResult,
                    3,
                    "/tmp/project/src-tauri/src/ai_talk.rs",
                ),
            ],
            last_meaningful_summary: None,
            ai_talk_context: None,
            turn_index: 0,
            turn_fingerprint: None,
            terminal_state: None,
            started_at: now,
            updated_at: now,
            started_at_ms: 1,
            updated_at_ms: 3,
        };

        assert_eq!(
            best_session_summary(&session).as_deref(),
            Some("详细细化一下")
        );
    }

    #[test]
    fn prioritized_events_keeps_all_high_priority_and_limits_low() {
        let mut events = vec![
            digest(EventType::Thinking, 1, "帮我优化逻辑"),
            digest(EventType::ToolUse, 2, "/src/a.rs"),
            digest(EventType::ToolResult, 3, "/src/a.rs"),
            digest(EventType::ToolUse, 4, "/src/b.rs"),
            digest(EventType::ToolResult, 5, "/src/b.rs"),
            digest(EventType::ToolUse, 6, "/src/c.rs"),
            digest(EventType::ToolResult, 7, "/src/c.rs"),
            digest(EventType::ToolUse, 8, "/src/d.rs"),
            digest(EventType::ToolResult, 9, "/src/d.rs"),
            digest(EventType::Thinking, 10, "再看看这个文件"),
            digest(EventType::ToolUse, 11, "/src/e.rs"),
            digest(EventType::Complete, 12, "优化完成"),
        ];
        // Mark all as informative (default in digest helper)
        for e in &mut events {
            e.informative = true;
        }

        let result = prioritized_events(&events);

        let high: Vec<_> = result
            .iter()
            .filter(|e| matches!(e.event_type, EventType::Thinking | EventType::Complete))
            .collect();
        assert_eq!(high.len(), 3, "all high-priority events preserved");

        let low: Vec<_> = result
            .iter()
            .filter(|e| matches!(e.event_type, EventType::ToolUse | EventType::ToolResult))
            .collect();
        assert!(low.len() <= 3, "low-priority events capped at 3");

        let timestamps: Vec<u64> = result.iter().map(|e| e.timestamp_ms).collect();
        let mut sorted = timestamps.clone();
        sorted.sort();
        assert_eq!(timestamps, sorted, "events sorted by timestamp");
    }

    #[test]
    fn prioritized_events_preserves_all_when_few_events() {
        let events = vec![
            digest(EventType::Thinking, 1, "用户意图"),
            digest(EventType::ToolUse, 2, "/src/a.rs"),
            digest(EventType::Complete, 3, "完成"),
        ];

        let result = prioritized_events(&events);
        assert_eq!(result.len(), 3, "all events kept when under limit");
    }
}
