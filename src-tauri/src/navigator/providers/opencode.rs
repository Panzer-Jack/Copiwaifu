use super::{shared, EventType};

pub fn normalize(raw_event: &str) -> Option<EventType> {
    shared::normalize(raw_event).or(match raw_event {
        "tool_started" => Some(EventType::ToolUse),
        "tool_finished" => Some(EventType::ToolResult),
        "tool_error" => Some(EventType::Error),
        "turn_completed" => Some(EventType::Complete),
        "question_requested" => Some(EventType::NeedsAttention),
        _ => None,
    })
}
