use std::collections::HashMap;
use tokio::sync::Mutex;

pub struct ActiveStreams(
    pub(crate) Mutex<HashMap<String, super::commands::agent_chat_streams::StreamEntry>>,
);
