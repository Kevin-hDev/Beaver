use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationAdmissionError(&'static str);

impl fmt::Display for ConversationAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for ConversationAdmissionError {}

pub(super) const fn error() -> ConversationAdmissionError {
    ConversationAdmissionError(super::conversation_admission::PUBLIC_ERROR_CODE)
}

pub(super) const fn capacity_error() -> ConversationAdmissionError {
    ConversationAdmissionError(super::session_limits::SESSION_CAPACITY_REACHED)
}
