use super::request_usage::CacheUsageStatus;
use super::request_usage_cache::ParsedCacheUsage;

pub(super) fn parse(value: &serde_json::Value) -> ParsedCacheUsage {
    let paths = [
        "/cache_read_input_tokens",
        "/cache_creation_input_tokens",
        "/cache_creation/ephemeral_5m_input_tokens",
        "/cache_creation/ephemeral_1h_input_tokens",
    ];
    let seen = paths.iter().any(|path| value.pointer(path).is_some());
    if !seen {
        return ParsedCacheUsage::default();
    }
    if paths
        .iter()
        .any(|path| value.pointer(path).is_some() && count(value, path).is_none())
    {
        return invalid();
    }
    let read = count(value, paths[0]);
    let detailed_seen = value.pointer(paths[2]).is_some() || value.pointer(paths[3]).is_some();
    let detailed_write = detailed_seen
        .then(|| {
            count(value, paths[2])
                .unwrap_or(0)
                .checked_add(count(value, paths[3]).unwrap_or(0))
                .filter(|total| *total <= super::MAX_REQUEST_TOKENS)
        })
        .flatten();
    let write = count(value, paths[1]).or(detailed_write);
    ParsedCacheUsage {
        read,
        write,
        status: CacheUsageStatus::Reported,
        ..Default::default()
    }
}

fn invalid() -> ParsedCacheUsage {
    ParsedCacheUsage {
        status: CacheUsageStatus::Invalid,
        ..Default::default()
    }
}

fn count(value: &serde_json::Value, path: &str) -> Option<u64> {
    value
        .pointer(path)
        .and_then(serde_json::Value::as_u64)
        .filter(|count| *count <= super::MAX_REQUEST_TOKENS)
}
