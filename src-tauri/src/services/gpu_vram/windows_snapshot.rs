pub(super) fn parse_sources(bytes: &[u8], truncated: bool) -> Option<(u64, u64)> {
    if truncated {
        return None;
    }
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.lines();
    let mut fields = lines.next()?.split(',').map(str::trim);
    let registry_total = fields.next()?.parse::<u64>().ok()?;
    let cim_total = fields.next()?.parse::<u64>().ok()?;
    let cim_used = fields.next()?.parse::<u64>().ok()?;
    let counter_total = fields.next()?.parse::<u64>().ok()?;
    let counter_used = fields.next()?.parse::<u64>().ok()?;
    if fields.next().is_some() || lines.any(|line| !line.trim().is_empty()) {
        return None;
    }
    let total = [registry_total, cim_total, counter_total]
        .into_iter()
        .find(|value| *value > 0)
        .unwrap_or(0);
    let used = [cim_used, counter_used]
        .into_iter()
        .find(|value| *value > 0)
        .unwrap_or(0);
    (total > 0 || used > 0).then_some((total / 1_048_576, used / 1_048_576))
}

#[cfg(test)]
mod tests {
    #[test]
    fn system_counter_values_recover_when_registry_and_cim_are_unavailable() {
        let snapshot = super::parse_sources(b"0,0,0,8589934592,2147483648\n", false);

        assert_eq!(snapshot, Some((8_192, 2_048)));
    }

    #[test]
    fn malformed_or_truncated_output_is_rejected() {
        assert_eq!(
            super::parse_sources(b"0,0,0,8589934592,2147483648\n", true),
            None
        );
        assert_eq!(super::parse_sources(b"0,0,invalid,0,0\n", false), None);
        assert_eq!(super::parse_sources(b"0,0,0,1,2,3\n", false), None);
    }
}
