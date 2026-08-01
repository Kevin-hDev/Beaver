use super::tool_bash_session::CompletionKind;

pub fn after_termination_attempt(
    completion: CompletionKind,
    termination_confirmed: bool,
) -> CompletionKind {
    if !termination_confirmed && matches!(completion, CompletionKind::Stopped) {
        CompletionKind::Failed
    } else {
        completion
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconfirmed_requested_stop_becomes_a_failure() {
        assert!(matches!(
            after_termination_attempt(CompletionKind::Stopped, false),
            CompletionKind::Failed
        ));
    }

    #[test]
    fn unconfirmed_termination_preserves_other_causes() {
        assert!(matches!(
            after_termination_attempt(CompletionKind::TimedOut, false),
            CompletionKind::TimedOut
        ));
        assert!(matches!(
            after_termination_attempt(CompletionKind::Cancelled, false),
            CompletionKind::Cancelled
        ));
        assert!(matches!(
            after_termination_attempt(CompletionKind::Failed, false),
            CompletionKind::Failed
        ));
    }
}
