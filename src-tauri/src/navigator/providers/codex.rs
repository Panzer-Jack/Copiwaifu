use super::{shared, EventType};

pub fn normalize(raw_event: &str) -> Option<EventType> {
    shared::normalize(raw_event).or(match raw_event {
        "notify" => Some(EventType::ToolUse),
        _ => None,
    })
}
