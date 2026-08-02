pub(super) fn metric_turn(zero_based_turn: usize) -> u32 {
    u32::try_from(zero_based_turn)
        .unwrap_or(u32::MAX)
        .saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::metric_turn;

    #[test]
    fn provider_metrics_use_a_one_based_turn_number() {
        assert_eq!(metric_turn(0), 1);
        assert_eq!(metric_turn(1), 2);
        assert_eq!(
            metric_turn(crate::services::agent_local::agent_loop_limits::MAX_TURNS - 1),
            crate::services::agent_local::agent_loop_limits::MAX_TURNS as u32,
        );
    }

    #[test]
    fn an_impossible_turn_cannot_wrap_back_to_zero() {
        assert_eq!(metric_turn(usize::MAX), u32::MAX);
    }
}
