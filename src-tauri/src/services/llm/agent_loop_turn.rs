pub(super) fn metric_turn(zero_based_turn: usize) -> u32 {
    u32::try_from(zero_based_turn)
        .unwrap_or(u32::MAX)
        .saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::metric_turn;

    #[test]
    fn metric_turn_supports_work_past_the_old_limit() {
        assert_eq!(metric_turn(250), 251);
        assert_eq!(metric_turn(usize::MAX), u32::MAX);
    }
}
