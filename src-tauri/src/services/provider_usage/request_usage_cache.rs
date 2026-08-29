use super::request_usage::{CacheMissSource, CacheUsageStatus};
use super::usage_context::{UsageApiFormat, UsageContext};

const MAX_REQUEST_TOKENS: u64 = 10_000_000_000;

#[derive(Debug, Default)]
pub(super) struct ParsedCacheUsage {
    pub read: Option<u64>,
    pub write: Option<u64>,
    pub miss: Option<u64>,
    pub miss_source: CacheMissSource,
    pub status: CacheUsageStatus,
}

pub(super) fn parse(
    value: &serde_json::Value,
    input: Option<u64>,
    context: UsageContext<'_>,
) -> ParsedCacheUsage {
    if context.canonical_provider_id == "deepseek"
        && context.api_format == UsageApiFormat::ChatCompletions
    {
        return parse_deepseek(value, input);
    }

    let read_paths = match context.api_format {
        UsageApiFormat::Responses => [
            "/input_tokens_details/cached_tokens",
            "/prompt_tokens_details/cached_tokens",
        ],
        UsageApiFormat::ChatCompletions | UsageApiFormat::GeminiNative => [
            "/prompt_tokens_details/cached_tokens",
            "/input_tokens_details/cached_tokens",
        ],
    };
    let cache_write_supported = context.canonical_provider_id == "openrouter"
        || (context.canonical_provider_id == "openai"
            && crate::services::llm::providers::openai::is_gpt_56(context.model));
    let mut parsed = ParsedCacheUsage {
        read: first_count(value, &read_paths),
        write: cache_write_supported
            .then(|| {
                first_count(
                    value,
                    &[
                        "/prompt_tokens_details/cache_write_tokens",
                        "/input_tokens_details/cache_write_tokens",
                    ],
                )
            })
            .flatten(),
        ..Default::default()
    };

    if context.canonical_provider_id == "moonshot" && parsed.read.is_none() {
        parsed.read = field_count(value, "cached_tokens");
    }
    if context.api_format == UsageApiFormat::GeminiNative && parsed.read.is_none() {
        parsed.read = field_count(value, "cachedContentTokenCount");
    }
    let mistral_cache_seen = context.canonical_provider_id == "mistral"
        && (read_paths
            .iter()
            .any(|path| value.pointer(path).is_some_and(|value| !value.is_null()))
            || value
                .pointer("/prompt_token_details/cached_tokens")
                .is_some_and(|value| !value.is_null())
            || value
                .get("num_cached_tokens")
                .is_some_and(|value| !value.is_null()));
    if context.canonical_provider_id == "mistral" && parsed.read.is_none() {
        parsed.read = first_count(
            value,
            &["/prompt_token_details/cached_tokens", "/num_cached_tokens"],
        );
        if !mistral_cache_seen && input.is_some() {
            parsed.read = Some(0);
        }
    }

    let cache_field_seen = read_paths.iter().any(|path| value.pointer(path).is_some())
        || (cache_write_supported
            && (value
                .pointer("/prompt_tokens_details/cache_write_tokens")
                .is_some()
                || value
                    .pointer("/input_tokens_details/cache_write_tokens")
                    .is_some()))
        || (context.canonical_provider_id == "moonshot" && value.get("cached_tokens").is_some())
        || (context.api_format == UsageApiFormat::GeminiNative
            && value.get("cachedContentTokenCount").is_some())
        || mistral_cache_seen
        || (context.canonical_provider_id == "mistral" && input.is_some());

    parsed.status = status_for(cache_field_seen, parsed.read, parsed.write, None, input);
    if context.canonical_provider_id == "mistral"
        && parsed.read.is_some_and(|tokens| tokens % 64 != 0)
    {
        parsed.status = CacheUsageStatus::Invalid;
    }
    if parsed.status == CacheUsageStatus::Invalid {
        parsed.read = None;
        parsed.write = None;
    } else if should_calculate_miss(context) {
        parsed.miss = input
            .zip(parsed.read)
            .and_then(|(total, read)| total.checked_sub(read));
        if parsed.miss.is_some() {
            parsed.miss_source = CacheMissSource::Calculated;
        }
    }
    parsed
}

fn parse_deepseek(value: &serde_json::Value, input: Option<u64>) -> ParsedCacheUsage {
    let hit_seen = value.get("prompt_cache_hit_tokens").is_some();
    let miss_seen = value.get("prompt_cache_miss_tokens").is_some();
    if !hit_seen && !miss_seen {
        return ParsedCacheUsage::default();
    }
    let read = field_count(value, "prompt_cache_hit_tokens");
    let miss = field_count(value, "prompt_cache_miss_tokens");
    let Some((read, miss)) = read.zip(miss) else {
        return ParsedCacheUsage {
            status: CacheUsageStatus::Invalid,
            ..Default::default()
        };
    };
    let total = read.saturating_add(miss);
    if total > MAX_REQUEST_TOKENS || input.is_some_and(|reported| reported != total) {
        return ParsedCacheUsage {
            status: CacheUsageStatus::Invalid,
            ..Default::default()
        };
    }
    ParsedCacheUsage {
        read: Some(read),
        miss: Some(miss),
        miss_source: CacheMissSource::Reported,
        status: CacheUsageStatus::Reported,
        ..Default::default()
    }
}

fn should_calculate_miss(context: UsageContext<'_>) -> bool {
    matches!(
        context.canonical_provider_id,
        "openai" | "openrouter" | "xai" | "mistral" | "cerebras" | "zai" | "moonshot"
    ) || context.api_format == UsageApiFormat::GeminiNative
        || (context.canonical_provider_id == "deepseek"
            && context.api_format == UsageApiFormat::Responses)
}

fn status_for(
    seen: bool,
    read: Option<u64>,
    write: Option<u64>,
    miss: Option<u64>,
    input: Option<u64>,
) -> CacheUsageStatus {
    if !seen {
        return CacheUsageStatus::Unknown;
    }
    let invalid = read.is_none() && write.is_none() && miss.is_none()
        || input.is_some_and(|total| {
            read.is_some_and(|count| count > total)
                || write.is_some_and(|count| count > total)
                || miss.is_some_and(|count| count > total)
                || read.unwrap_or(0).saturating_add(write.unwrap_or(0)) > total
        });
    if invalid {
        CacheUsageStatus::Invalid
    } else {
        CacheUsageStatus::Reported
    }
}

fn first_count(value: &serde_json::Value, paths: &[&str]) -> Option<u64> {
    paths.iter().find_map(|path| {
        value
            .pointer(path)
            .and_then(serde_json::Value::as_u64)
            .filter(|count| *count <= MAX_REQUEST_TOKENS)
    })
}

fn field_count(value: &serde_json::Value, field: &str) -> Option<u64> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .filter(|count| *count <= MAX_REQUEST_TOKENS)
}
