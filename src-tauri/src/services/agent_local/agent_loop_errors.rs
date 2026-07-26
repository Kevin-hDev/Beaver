pub fn max_turns_message() -> String {
    format!(
        "Limite de tours agent atteinte ({}).",
        super::agent_loop_limits::MAX_TURNS
    )
}
