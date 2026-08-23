const BYTES_PER_MB: u64 = 1_048_576;
const MAX_ADAPTER_ROWS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AdapterLuid {
    pub(super) high: i32,
    pub(super) low: u32,
}

impl AdapterLuid {
    pub(super) const fn new(high: i32, low: u32) -> Self {
        Self { high, low }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AdapterCapacity {
    pub(super) luid: AdapterLuid,
    pub(super) total_bytes: u64,
    pub(super) software: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AdapterUsage {
    pub(super) luid: AdapterLuid,
    pub(super) used_bytes: u64,
}

pub(super) fn parse_usage_rows(bytes: &[u8], truncated: bool) -> Option<Vec<AdapterUsage>> {
    if truncated {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if rows.len() >= MAX_ADAPTER_ROWS {
            return None;
        }
        let mut fields = line.split(',').map(str::trim);
        let luid = parse_luid(fields.next()?)?;
        let used_bytes = fields.next()?.parse::<u64>().ok()?;
        if fields.next().is_some() {
            return None;
        }
        rows.push(AdapterUsage { luid, used_bytes });
    }
    (!rows.is_empty()).then_some(rows)
}

pub(super) fn select_snapshot(
    capacities: &[AdapterCapacity],
    usages: &[AdapterUsage],
) -> Option<(u64, Option<u64>)> {
    let selected = capacities
        .iter()
        .filter(|adapter| !adapter.software && adapter.total_bytes > 0)
        .max_by_key(|adapter| (adapter.total_bytes, adapter.luid.high, adapter.luid.low))?;
    let matching = usages
        .iter()
        .filter(|usage| usage.luid == selected.luid)
        .map(|usage| usage.used_bytes)
        .reduce(u64::saturating_add);
    Some((
        selected.total_bytes / BYTES_PER_MB,
        matching.map(|used| used / BYTES_PER_MB),
    ))
}

fn parse_luid(name: &str) -> Option<AdapterLuid> {
    let mut fields = name.strip_prefix("luid_")?.split('_');
    let high = parse_hex_u32(fields.next()?)? as i32;
    let low = parse_hex_u32(fields.next()?)?;
    if fields.next()? != "phys" || fields.next()?.parse::<u32>().is_err() {
        return None;
    }
    fields
        .next()
        .is_none()
        .then_some(AdapterLuid::new(high, low))
}

fn parse_hex_u32(value: &str) -> Option<u32> {
    u32::from_str_radix(value.strip_prefix("0x")?, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::{AdapterCapacity, AdapterLuid, AdapterUsage};

    #[test]
    fn parses_real_gpu_adapter_memory_rows_by_luid() {
        let rows = super::parse_usage_rows(
            b"luid_0x00000000_0x0000FDA5_phys_0,1490948096\n\
luid_0x00000000_0x00011DD2_phys_0,0\n",
            false,
        );

        assert_eq!(
            rows,
            Some(vec![
                AdapterUsage {
                    luid: AdapterLuid::new(0, 0x0000_FDA5),
                    used_bytes: 1_490_948_096,
                },
                AdapterUsage {
                    luid: AdapterLuid::new(0, 0x0001_1DD2),
                    used_bytes: 0,
                },
            ])
        );
    }

    #[test]
    fn rejects_malformed_truncated_or_unbounded_rows() {
        assert_eq!(
            super::parse_usage_rows(b"luid_0x0_0x1_phys_0,12\n", true),
            None
        );
        assert_eq!(super::parse_usage_rows(b"bad,12\n", false), None);
        let oversized = b"luid_0x00000000_0x00000001_phys_0,1\n".repeat(65);
        assert_eq!(super::parse_usage_rows(&oversized, false), None);
    }

    #[test]
    fn selects_capacity_and_usage_only_for_the_same_adapter() {
        let capacities = [
            AdapterCapacity {
                luid: AdapterLuid::new(0, 0x0001_1DD2),
                total_bytes: 512 * 1_048_576,
                software: false,
            },
            AdapterCapacity {
                luid: AdapterLuid::new(0, 0x0000_FDA5),
                total_bytes: 16_384 * 1_048_576,
                software: false,
            },
        ];
        let usages = [AdapterUsage {
            luid: AdapterLuid::new(0, 0x0000_FDA5),
            used_bytes: 1_422 * 1_048_576,
        }];

        assert_eq!(
            super::select_snapshot(&capacities, &usages),
            Some((16_384, Some(1_422)))
        );
    }

    #[test]
    fn mismatched_or_software_adapters_never_fabricate_a_percentage() {
        let capacities = [
            AdapterCapacity {
                luid: AdapterLuid::new(0, 7),
                total_bytes: 32_768 * 1_048_576,
                software: true,
            },
            AdapterCapacity {
                luid: AdapterLuid::new(0, 8),
                total_bytes: 8_192 * 1_048_576,
                software: false,
            },
        ];
        let usages = [AdapterUsage {
            luid: AdapterLuid::new(0, 9),
            used_bytes: 4_096 * 1_048_576,
        }];

        assert_eq!(
            super::select_snapshot(&capacities, &usages),
            Some((8_192, None))
        );
    }
}
