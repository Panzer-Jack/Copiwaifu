use super::{shared, EventType};

pub fn normalize(raw_event: &str) -> Option<EventType> {
    shared::normalize(raw_event).or(match raw_event {
        "BeforeTool" | "BeforeAgent" => Some(EventType::ToolUse),
        "AfterTool" | "AfterAgent" => Some(EventType::ToolResult),
        _ => None,
    })
}
