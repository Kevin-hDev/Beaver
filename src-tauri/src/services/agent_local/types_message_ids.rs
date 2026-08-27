use super::types_message::{AgentMessage, ToolCallRequest};

impl ToolCallRequest {
    pub fn local_id() -> String {
        format!("local-call-{}", uuid::Uuid::new_v4())
    }
}

impl AgentMessage {
    pub fn new_turn_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
