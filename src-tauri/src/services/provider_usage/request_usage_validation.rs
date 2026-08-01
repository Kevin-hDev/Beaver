use super::request_usage::{
    CacheMissSource, CacheUsageStatus, RequestUsage, MAX_COST_USD, MAX_REQUEST_TOKENS,
};

pub(super) fn is_valid(usage: &RequestUsage) -> bool {
    let counts = [
        usage.input_tokens,
        usage.output_tokens,
        usage.cached_input_tokens,
        usage.cache_write_input_tokens,
        usage.cache_miss_input_tokens,
        usage.reasoning_output_tokens,
        usage.total_tokens,
    ];
    counts
        .into_iter()
        .flatten()
        .all(|value| value <= MAX_REQUEST_TOKENS)
        && valid_token_relations(usage)
        && valid_cache_state(usage)
        && usage
            .exact_cost_usd_micros
            .is_none_or(|micros| micros <= (MAX_COST_USD * 1_000_000.0) as u64)
}

fn valid_token_relations(usage: &RequestUsage) -> bool {
    usage
        .cached_input_tokens
        .zip(usage.input_tokens)
        .is_none_or(|(cached, input)| cached <= input)
        && usage
            .cache_write_input_tokens
            .zip(usage.input_tokens)
            .is_none_or(|(written, input)| written <= input)
        && usage
            .cache_miss_input_tokens
            .zip(usage.input_tokens)
            .is_none_or(|(miss, input)| miss <= input)
        && usage.input_tokens.is_none_or(|input| {
            usage
                .cached_input_tokens
                .unwrap_or(0)
                .saturating_add(usage.cache_write_input_tokens.unwrap_or(0))
                <= input
        })
        && usage
            .reasoning_output_tokens
            .zip(usage.output_tokens)
            .is_none_or(|(reasoning, output)| reasoning <= output)
}

fn valid_cache_state(usage: &RequestUsage) -> bool {
    let has_cache_value = usage.cached_input_tokens.is_some()
        || usage.cache_write_input_tokens.is_some()
        || usage.cache_miss_input_tokens.is_some();
    let state_is_coherent = match usage.cache_status {
        CacheUsageStatus::Unknown | CacheUsageStatus::Invalid => {
            !has_cache_value && usage.cache_miss_source == CacheMissSource::Unknown
        }
        CacheUsageStatus::Reported => {
            has_cache_value
                && matches!(
                    (usage.cache_miss_input_tokens, usage.cache_miss_source),
                    (None, CacheMissSource::Unknown)
                        | (
                            Some(_),
                            CacheMissSource::Reported | CacheMissSource::Calculated
                        )
                )
        }
    };
    state_is_coherent && valid_miss_relation(usage)
}

fn valid_miss_relation(usage: &RequestUsage) -> bool {
    match usage.cache_miss_source {
        CacheMissSource::Unknown => usage.cache_miss_input_tokens.is_none(),
        CacheMissSource::Calculated => {
            usage
                .input_tokens
                .zip(usage.cached_input_tokens)
                .and_then(|(input, cached)| input.checked_sub(cached))
                == usage.cache_miss_input_tokens
        }
        CacheMissSource::Reported => usage
            .input_tokens
            .zip(usage.cached_input_tokens)
            .zip(usage.cache_miss_input_tokens)
            .is_none_or(|((input, cached), miss)| cached.saturating_add(miss) == input),
    }
}
