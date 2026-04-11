use super::{shared, EventType};

pub fn normalize(raw_event: &str) -> Option<EventType> {
    shared::normalize(raw_event).or(match raw_event {
        "agentStop" => Some(EventType::Complete),
        _ => None,
    })
}
