use super::types_session::AgentSession;

pub(crate) fn normalize(session: &mut AgentSession) {
    for run in &mut session.diagnostic_runs {
        if run
            .error_type
            .as_deref()
            .is_some_and(|value| value != "unknown")
        {
            continue;
        }
        let Some((code, message)) = run
            .events
            .iter()
            .rev()
            .find_map(|event| loop_code(&event.message).map(|code| (code, event.message.clone())))
            .or_else(|| {
                run.safe_summary
                    .as_deref()
                    .and_then(loop_code)
                    .map(|code| (code, run.safe_summary.clone().unwrap_or_default()))
            })
        else {
            continue;
        };
        run.error_type = Some(code.to_string());
        for event in &mut run.events {
            if event
                .error_type
                .as_deref()
                .is_none_or(|value| value == "unknown")
                && loop_code(&event.message) == Some(code)
            {
                event.error_type = Some(code.to_string());
            }
        }
        run.safe_summary = Some(super::stream_diagnostics_failure::safe_summary(
            run, code, &message,
        ));
    }
}

fn loop_code(message: &str) -> Option<&'static str> {
    let lower = message.to_lowercase();
    if lower.contains("limite de tours") {
        Some("max_turns")
    } else if lower.contains("répété") || lower.contains("circuit") {
        Some("circuit_breaker")
    } else {
        None
    }
}
