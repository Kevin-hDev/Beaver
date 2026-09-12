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

pub(super) fn recovery_error(code: &str) -> ConversationAdmissionError {
    if code == super::session_limits::SESSION_CAPACITY_REACHED {
        capacity_error()
    } else {
        error()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn recovery_capacity_keeps_its_public_code() {
        assert_eq!(
            super::recovery_error(super::super::session_limits::SESSION_CAPACITY_REACHED)
                .to_string(),
            "session_capacity_reached"
        );
        assert_eq!(
            super::recovery_error("private recovery detail").to_string(),
            super::super::conversation_admission::PUBLIC_ERROR_CODE
        );
    }
}
