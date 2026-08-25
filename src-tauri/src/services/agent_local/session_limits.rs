use std::io::{self, Write};

use serde::Serialize;

use super::types_session::AgentSession;

pub const CURRENT_SESSION_SCHEMA_VERSION: u16 = 2;
pub const MAX_SESSION_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_SESSION_FILES: usize = 4_096;
pub const MAX_MESSAGES_PER_SESSION: usize = 2_000;

pub fn validate_continuity(session: &AgentSession) -> Result<(), String> {
    let maximum = crate::services::reasoning_continuity::limits::MAX_SESSION_CONTINUITY_BYTES;
    let mut writer = LimitedCounter::new(maximum);
    for continuation in session
        .messages
        .iter()
        .filter_map(|message| message.continuation.as_ref())
    {
        continuation
            .validate()
            .map_err(|_| invalid_session())?;
        continuation
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .map_err(|_| invalid_session())?;
    }
    Ok(())
}

pub fn validate_serialized_size(bytes: usize) -> Result<(), String> {
    (bytes as u64 <= MAX_SESSION_FILE_BYTES)
        .then_some(())
        .ok_or_else(save_failed)
}

pub fn invalid_session() -> String {
    "Session invalide".to_string()
}

pub fn save_failed() -> String {
    "Sauvegarde de session impossible".to_string()
}

struct LimitedCounter {
    written: usize,
    maximum: usize,
}

impl LimitedCounter {
    const fn new(maximum: usize) -> Self {
        Self {
            written: 0,
            maximum,
        }
    }
}

impl Write for LimitedCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .written
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("session_continuity_limit"))?;
        if next > self.maximum {
            return Err(io::Error::other("session_continuity_limit"));
        }
        self.written = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
