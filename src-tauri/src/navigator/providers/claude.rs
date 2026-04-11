use super::{shared, EventType};

pub fn normalize(raw_event: &str) -> Option<EventType> {
    shared::normalize(raw_event).or(match raw_event {
        "Elicitation" | "SubagentStart" | "subagentStart" | "SubagentStop" | "subagentStop"
        | "PreCompact" | "preCompact" | "PostCompact" | "WorktreeCreate" => {
            Some(EventType::ToolUse)
        }
        _ => None,
    })
}
