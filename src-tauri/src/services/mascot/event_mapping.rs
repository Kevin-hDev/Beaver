use super::MascotAnimation;
use crate::services::agent_local::types_ollama::StreamEvent;
use std::time::Duration;

pub(super) const SUCCESS_DURATION: Duration = Duration::from_millis(2200);
pub(super) const FAILURE_DURATION: Duration = Duration::from_millis(2600);
const ALERT_DURATION: Duration = Duration::from_millis(1800);

pub(super) fn animation_for_event(
    event: &StreamEvent,
) -> Option<(MascotAnimation, Option<Duration>, bool)> {
    let persistent = |animation| Some((animation, None, false));
    match event {
        StreamEvent::Thinking { .. }
        | StreamEvent::Token { .. }
        | StreamEvent::ContentPhase { .. } => persistent(MascotAnimation::Thinking),
        StreamEvent::ToolCall { name, .. } | StreamEvent::ToolResult { name, .. } => {
            persistent(tool_animation(name))
        }
        StreamEvent::Compressing { .. } | StreamEvent::SubagentSpawned { .. } => {
            persistent(MascotAnimation::WorkLaptop)
        }
        StreamEvent::PermissionRequest(..) | StreamEvent::InteractiveChoiceRequest { .. } => {
            persistent(MascotAnimation::Waiting)
        }
        StreamEvent::Done { .. } => Some((MascotAnimation::Success, Some(SUCCESS_DURATION), false)),
        StreamEvent::Error { .. } => Some((MascotAnimation::Failed, Some(FAILURE_DURATION), false)),
        StreamEvent::RetryIndicator { .. } | StreamEvent::Notice { .. } => {
            Some((MascotAnimation::Alert, Some(ALERT_DURATION), true))
        }
        _ => None,
    }
}

fn tool_animation(name: &str) -> MascotAnimation {
    match name {
        "read_file" | "read_document" | "read_spreadsheet" | "list_dir" | "grep" | "glob"
        | "web_search" | "web_fetch" => MascotAnimation::ExploreBook,
        _ => MascotAnimation::WorkLaptop,
    }
}

#[cfg(test)]
#[path = "mapping_tests.rs"]
mod tests;
