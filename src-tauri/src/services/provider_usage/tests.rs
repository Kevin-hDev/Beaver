use super::request_usage::{CacheMissSource, CacheUsageStatus, RequestUsage};
use super::snapshot::build_snapshot;
use super::types::{LocalSnapshot, RemoteData};
use super::usage_context::{UsageApiFormat, UsageContext};
use serde_json::json;

#[tokio::test]
async fn gemini_cache_writes_require_reported_cost_even_with_catalog_pricing() {
    use crate::services::llm::litellm_catalog;
    let key = "openrouter/google/gemini-3.8-flash";
    let mut entries = litellm_catalog::parse_catalog(
        r#"{
        "openrouter/google/gemini-3.8-flash":{"litellm_provider":"openrouter","mode":"chat",
        "input_cost_per_token":0.00000075,"output_cost_per_token":0.00000375,
        "cache_creation_input_token_cost":0.0000000416667}
    }"#,
    );
    let replacement = entries.remove(key).unwrap();
    let previous = {
        let mut catalog = litellm_catalog::get_lock().write().await;
        let previous = catalog.get(key).cloned();
        let mut priced = previous.clone().unwrap_or_else(|| replacement.clone());
        priced.input_cost_per_token = replacement.input_cost_per_token;
        priced.output_cost_per_token = replacement.output_cost_per_token;
        priced.cache_creation_input_token_cost = replacement.cache_creation_input_token_cost;
        catalog.insert(key.into(), priced);
        previous
    };
    let usage = RequestUsage {
        input_tokens: Some(10000),
        output_tokens: Some(10),
        cached_input_tokens: Some(4100),
        cache_write_input_tokens: Some(4100),
        ..Default::default()
    };
    let estimated = super::pricing::resolve("openrouter", "google/gemini-3.8-flash", &usage).await;
    let exact = super::pricing::resolve(
        "openrouter",
        "google/gemini-3.8-flash",
        &RequestUsage {
            exact_cost_usd_micros: Some(739),
            ..usage
        },
    )
    .await;
    {
        let mut catalog = litellm_catalog::get_lock().write().await;
        if let Some(old) = previous {
            catalog.insert(key.into(), old);
        } else {
            catalog.remove(key);
        }
    }
    assert_eq!(estimated.micros, None);
    assert_eq!(exact.micros, Some(739));
    assert!(exact.exact);
}

#[test]
fn gemini_explicit_cache_can_write_and_read_the_same_prefix() {
    let value = json!({"prompt_tokens":5618,"completion_tokens":19,
        "prompt_tokens_details":{"cached_tokens":5599,"cache_write_tokens":5599}});
    let usage = RequestUsage::from_json_with_context(
        &value,
        UsageContext::chat("openrouter", "google/gemini-3.8-flash"),
    )
    .unwrap();
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
    assert_eq!(usage.cached_input_tokens, Some(5599));
    assert_eq!(usage.cache_write_input_tokens, Some(5599));
    assert_eq!(usage.cache_miss_input_tokens, Some(19));
    for model in ["openai/gpt-6-astra", "z-ai/glm-5.3-flash"] {
        let usage =
            RequestUsage::from_json_with_context(&value, UsageContext::chat("openrouter", model))
                .unwrap();
        assert_eq!(usage.cache_status, CacheUsageStatus::Invalid);
    }
    let usage = RequestUsage::from_json_with_context(
        &json!({"prompt_tokens":100,"prompt_tokens_details":{"cached_tokens":101,"cache_write_tokens":0}}),
        UsageContext::chat("openrouter","google/gemini-3.8-flash")
    ).unwrap();
    assert_eq!(usage.cache_status, CacheUsageStatus::Invalid);
}

#[test]
fn openai_reasoning_is_not_added_twice() {
    let usage = RequestUsage::from_json(&json!({
        "prompt_tokens": 20,
        "completion_tokens": 12,
        "completion_tokens_details": { "reasoning_tokens": 8 },
        "total_tokens": 32
    }))
    .unwrap();

    assert_eq!(usage.output_tokens, Some(12));
    assert_eq!(usage.reasoning_output_tokens, Some(8));
    assert_eq!(usage.total_tokens, Some(32));
}

#[test]
fn gemini_thoughts_are_included_in_output_total() {
    let usage = RequestUsage::from_json(&json!({
        "promptTokenCount": 10,
        "candidatesTokenCount": 4,
        "thoughtsTokenCount": 6,
        "totalTokenCount": 20
    }))
    .unwrap();

    assert_eq!(usage.output_tokens, Some(10));
    assert_eq!(usage.reasoning_output_tokens, Some(6));
}

#[test]
fn exact_cost_accepts_a_bounded_decimal_string() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 2,
            "completion_tokens": 1,
            "cost": "0.000123"
        }),
        UsageContext::chat("openrouter", "openai/gpt-5.6"),
    )
    .unwrap();
    assert_eq!(usage.exact_cost_usd_micros, Some(123));
}

#[test]
fn undocumented_generic_cost_is_ignored() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 2,
            "completion_tokens": 1,
            "cost": "12.50"
        }),
        UsageContext::chat("moonshot", "kimi-k2.7-code"),
    )
    .unwrap();

    assert_eq!(usage.exact_cost_usd_micros, None);
}

#[test]
fn xai_integer_ticks_are_converted_without_floating_point() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 2,
            "completion_tokens": 1,
            "cost_in_usd_ticks": 1_234_560
        }),
        UsageContext::chat("xai", "grok-4.5"),
    )
    .unwrap();

    assert_eq!(usage.exact_cost_usd_micros, Some(123));
}

#[test]
fn cache_or_reasoning_data_alone_is_real_usage() {
    let usage = RequestUsage {
        cached_input_tokens: Some(4),
        reasoning_output_tokens: Some(2),
        ..Default::default()
    };
    assert!(!usage.is_empty());
}

#[test]
fn aggregate_counts_cache_observations_separately() {
    let mut breakdown = super::types::UsageBreakdown::default();
    let usage = RequestUsage {
        input_tokens: Some(100),
        output_tokens: Some(10),
        cached_input_tokens: Some(0),
        ..Default::default()
    };

    super::ledger_aggregate::add(
        &mut breakdown,
        super::UsageOrigin::ManualChat,
        super::UsageWorkload::Primary,
        &usage,
        Default::default(),
    );

    assert_eq!(breakdown.totals.cache_read_request_count, 1);
    assert_eq!(breakdown.totals.cache_write_request_count, 0);
    assert_eq!(breakdown.totals.cache_miss_request_count, 0);
}

#[test]
fn deepseek_keeps_reported_hits_and_misses_when_coherent() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 100,
            "prompt_cache_hit_tokens": 64,
            "prompt_cache_miss_tokens": 36
        }),
        UsageContext::chat("deepseek", "deepseek-v4-pro"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(64));
    assert_eq!(usage.cache_miss_input_tokens, Some(36));
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn deepseek_rejects_incoherent_cache_counts_without_clamping() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 100,
            "prompt_cache_hit_tokens": 90,
            "prompt_cache_miss_tokens": 20
        }),
        UsageContext::chat("deepseek", "deepseek-v4-pro"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, None);
    assert_eq!(usage.cache_miss_input_tokens, None);
    assert_eq!(usage.cache_status, CacheUsageStatus::Invalid);
}

#[test]
fn gpt_56_reads_both_cache_directions() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 1200,
            "prompt_tokens_details": {
                "cached_tokens": 800,
                "cache_write_tokens": 400
            }
        }),
        UsageContext::chat("openai", "gpt-5.6-sol"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(800));
    assert_eq!(usage.cache_write_input_tokens, Some(400));
    assert_eq!(usage.cache_miss_input_tokens, Some(400));
    assert_eq!(usage.cache_miss_source, CacheMissSource::Calculated);
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn google_chat_usage_reads_total_cached_tokens_without_double_counting_input() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 1200,
            "completion_tokens": 20,
            "total_cached_tokens": 800
        }),
        UsageContext::chat("google", "gemini-3.8-flash"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(800));
    assert_eq!(usage.cache_miss_input_tokens, Some(400));
    assert_eq!(usage.cache_miss_source, CacheMissSource::Calculated);
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
    assert_eq!(usage.input_tokens, Some(1200));
}

#[test]
fn astra_responses_usage_reads_cache_writes() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "input_tokens": 1200,
            "output_tokens": 20,
            "input_tokens_details": {
                "cached_tokens": 800,
                "cache_write_tokens": 400
            }
        }),
        UsageContext::responses("openai", "gpt-6-astra"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(800));
    assert_eq!(usage.cache_write_input_tokens, Some(400));
    assert_eq!(usage.cache_miss_input_tokens, Some(400));
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn google_cache_counter_distinguishes_absent_zero_and_invalid() {
    let absent = RequestUsage::from_json_with_context(
        &json!({"prompt_tokens": 120}),
        UsageContext::chat("google", "gemini-3.8-flash"),
    )
    .unwrap();
    assert_eq!(absent.cached_input_tokens, None);
    assert_eq!(absent.cache_status, CacheUsageStatus::Unknown);

    let zero = RequestUsage::from_json_with_context(
        &json!({"prompt_tokens": 120, "total_cached_tokens": 0}),
        UsageContext::chat("google", "gemini-3.8-flash"),
    )
    .unwrap();
    assert_eq!(zero.cached_input_tokens, Some(0));
    assert_eq!(zero.cache_miss_input_tokens, Some(120));
    assert_eq!(zero.cache_status, CacheUsageStatus::Reported);

    let invalid = RequestUsage::from_json_with_context(
        &json!({"prompt_tokens": 120, "total_cached_tokens": 121}),
        UsageContext::chat("google", "gemini-3.8-flash"),
    )
    .unwrap();
    assert_eq!(invalid.cached_input_tokens, None);
    assert_eq!(invalid.cache_miss_input_tokens, None);
    assert_eq!(invalid.cache_status, CacheUsageStatus::Invalid);
}

#[test]
fn google_total_cached_tokens_belongs_only_to_its_chat_contract() {
    for context in [
        UsageContext::responses("google", "gemini-3.8-flash"),
        UsageContext {
            canonical_provider_id: "google",
            model: "gemini-3.8-flash",
            api_format: UsageApiFormat::GeminiNative,
        },
        UsageContext::chat("zai", "glm-5.3-flash"),
    ] {
        let usage = RequestUsage::from_json_with_context(
            &json!({"prompt_tokens":120, "total_cached_tokens":80}),
            context,
        )
        .unwrap();
        assert_eq!(usage.cached_input_tokens, None);
        assert_eq!(usage.cache_status, CacheUsageStatus::Unknown);
    }
}

#[test]
fn qwen_reads_exact_cache_hit_and_creation_counters() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 1200,
            "prompt_tokens_details": {
                "cached_tokens": 800,
                "cache_creation_input_tokens": 400
            }
        }),
        UsageContext::chat("qwen", "qwen3.8-flash"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(800));
    assert_eq!(usage.cache_write_input_tokens, Some(400));
    assert_eq!(usage.cache_miss_input_tokens, None);
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn gpt_56_responses_uses_input_token_details() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "input_tokens": 2048,
            "input_tokens_details": {
                "cached_tokens": 1024,
                "cache_write_tokens": 512
            }
        }),
        UsageContext::responses("openai", "gpt-5.6-terra"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(1024));
    assert_eq!(usage.cache_write_input_tokens, Some(512));
    assert_eq!(usage.cache_miss_input_tokens, Some(1024));
    assert_eq!(usage.cache_miss_source, CacheMissSource::Calculated);
}

#[test]
fn anthropic_messages_reads_the_native_cache_hit_counter() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "input_tokens": 120,
            "cache_read_input_tokens": 80
        }),
        UsageContext {
            canonical_provider_id: "anthropic",
            model: "claude-haiku-4-5-20251001",
            api_format: UsageApiFormat::AnthropicMessages,
        },
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(80));
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn anthropic_messages_reads_both_cache_creation_windows_without_double_counting() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "input_tokens": 120,
            "cache_read_input_tokens": 80,
            "cache_creation_input_tokens": 30,
            "cache_creation": {
                "ephemeral_5m_input_tokens": 20,
                "ephemeral_1h_input_tokens": 10
            }
        }),
        UsageContext {
            canonical_provider_id: "anthropic",
            model: "claude-haiku-4-5-20251001",
            api_format: UsageApiFormat::AnthropicMessages,
        },
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(80));
    assert_eq!(usage.cache_write_input_tokens, Some(30));
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn anthropic_cache_creation_accepts_one_window_and_rejects_oversized_counts() {
    let context = UsageContext {
        canonical_provider_id: "anthropic",
        model: "claude-haiku-4-5-20251001",
        api_format: UsageApiFormat::AnthropicMessages,
    };
    let one_window = RequestUsage::from_json_with_context(
        &json!({
            "input_tokens": 10,
            "cache_creation": {"ephemeral_5m_input_tokens": 6}
        }),
        context,
    )
    .unwrap();
    assert_eq!(one_window.cache_write_input_tokens, Some(6));

    let invalid = RequestUsage::from_json_with_context(
        &json!({"cache_creation_input_tokens": 10_000_000_001_u64}),
        context,
    )
    .unwrap();
    assert_eq!(invalid.cache_status, CacheUsageStatus::Invalid);
    assert_eq!(invalid.cache_write_input_tokens, None);
}

#[test]
fn older_openai_models_ignore_gpt_56_only_write_counts() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 1200,
            "prompt_tokens_details": { "cache_write_tokens": 400 }
        }),
        UsageContext::chat("openai", "gpt-4o"),
    )
    .unwrap();

    assert_eq!(usage.cache_write_input_tokens, None);
    assert_eq!(usage.cache_status, CacheUsageStatus::Unknown);
}

#[test]
fn moonshot_flat_cache_count_is_route_specific() {
    let body = json!({ "prompt_tokens": 100, "cached_tokens": 80 });
    let moonshot =
        RequestUsage::from_json_with_context(&body, UsageContext::chat("moonshot", "kimi-k2.5"))
            .unwrap();
    let unknown = RequestUsage::from_json(&body).unwrap();

    assert_eq!(moonshot.cached_input_tokens, Some(80));
    assert_eq!(moonshot.cache_miss_input_tokens, Some(20));
    assert_eq!(moonshot.cache_miss_source, CacheMissSource::Calculated);
    assert_eq!(unknown.cached_input_tokens, None);
}

#[test]
fn moonshot_calculates_misses_only_from_an_explicit_valid_cache_count() {
    let zero = RequestUsage::from_json_with_context(
        &json!({ "prompt_tokens": 100, "cached_tokens": 0 }),
        UsageContext::chat("moonshot", "kimi-k3"),
    )
    .unwrap();
    assert_eq!(zero.cache_miss_input_tokens, Some(100));
    assert_eq!(zero.cache_miss_source, CacheMissSource::Calculated);

    let absent = RequestUsage::from_json_with_context(
        &json!({ "prompt_tokens": 100 }),
        UsageContext::chat("moonshot", "kimi-k3"),
    )
    .unwrap();
    assert_eq!(absent.cache_miss_input_tokens, None);
    assert_eq!(absent.cache_miss_source, CacheMissSource::Unknown);

    let invalid = RequestUsage::from_json_with_context(
        &json!({ "prompt_tokens": 100, "cached_tokens": 101 }),
        UsageContext::chat("moonshot", "kimi-k3"),
    )
    .unwrap();
    assert_eq!(invalid.cached_input_tokens, None);
    assert_eq!(invalid.cache_miss_input_tokens, None);
    assert_eq!(invalid.cache_status, CacheUsageStatus::Invalid);
}

#[test]
fn standard_chat_cache_paths_are_provider_aware_and_bounded() {
    for provider in ["openai", "cerebras", "zai"] {
        let usage = RequestUsage::from_json_with_context(
            &json!({
                "prompt_tokens": 128,
                "prompt_tokens_details": { "cached_tokens": 64 }
            }),
            UsageContext::chat(provider, "model"),
        )
        .unwrap();
        assert_eq!(usage.cached_input_tokens, Some(64), "{provider}");
        assert_eq!(usage.cache_miss_input_tokens, Some(64), "{provider}");
        assert_eq!(usage.cache_status, CacheUsageStatus::Reported, "{provider}");
    }

    let zero = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 128,
            "prompt_tokens_details": { "cached_tokens": 0 }
        }),
        UsageContext::chat("openai", "gpt-5.6-luna"),
    )
    .unwrap();
    assert_eq!(zero.cached_input_tokens, Some(0));
    assert_eq!(zero.cache_status, CacheUsageStatus::Reported);

    let absent = RequestUsage::from_json_with_context(
        &json!({ "prompt_tokens": 128 }),
        UsageContext::chat("openai", "gpt-5.6-luna"),
    )
    .unwrap();
    assert_eq!(absent.cached_input_tokens, None);
    assert_eq!(absent.cache_status, CacheUsageStatus::Unknown);

    for cached in [json!(null), json!("64"), json!(10_000_000_001_u64)] {
        let usage = RequestUsage::from_json_with_context(
            &json!({
                "prompt_tokens": 128,
                "prompt_tokens_details": { "cached_tokens": cached }
            }),
            UsageContext::chat("openai", "gpt-5.6-luna"),
        )
        .unwrap();
        assert_eq!(usage.cached_input_tokens, None);
        assert_eq!(usage.cache_status, CacheUsageStatus::Invalid);
    }
}

#[test]
fn mistral_documented_absence_means_a_zero_cache_hit() {
    let usage = RequestUsage::from_json_with_context(
        &json!({ "prompt_tokens": 96, "completion_tokens": 4 }),
        UsageContext::chat("mistral", "mistral-large"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(0));
    assert_eq!(usage.cache_miss_input_tokens, Some(96));
    assert_eq!(usage.cache_miss_source, CacheMissSource::Calculated);
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn mistral_nullable_cache_count_means_no_hit() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 96,
            "completion_tokens": 4,
            "num_cached_tokens": null
        }),
        UsageContext::chat("mistral", "mistral-large"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, Some(0));
    assert_eq!(usage.cache_miss_input_tokens, Some(96));
    assert_eq!(usage.cache_status, CacheUsageStatus::Reported);
}

#[test]
fn mistral_rejects_non_block_aligned_cache_hits() {
    let usage = RequestUsage::from_json_with_context(
        &json!({
            "prompt_tokens": 100,
            "prompt_tokens_details": { "cached_tokens": 65 }
        }),
        UsageContext::chat("mistral", "mistral-large"),
    )
    .unwrap();

    assert_eq!(usage.cached_input_tokens, None);
    assert_eq!(usage.cache_status, CacheUsageStatus::Invalid);
}

#[test]
fn invalid_connection_is_rejected() {
    assert!(super::types::validate_connection_id("../secret").is_err());
    assert!(super::types::validate_connection_id("openai").is_ok());
    assert!(super::types::validate_connection_id("anthropic").is_ok());
    assert!(super::types::validate_connection_id("qwen").is_ok());
}

#[test]
fn snapshot_keeps_remote_timestamp() {
    let remote = RemoteData {
        fetched_at: 123,
        ..Default::default()
    };
    let snapshot = build_snapshot(
        "xai-oauth",
        LocalSnapshot::default(),
        remote,
        Default::default(),
    );

    assert_eq!(snapshot.canonical_provider_id, "xai");
    assert_eq!(snapshot.auth_source, "oauth");
    assert_eq!(snapshot.refreshed_at, 123);
}

#[tokio::test]
async fn exact_provider_cost_wins() {
    let usage = RequestUsage {
        exact_cost_usd_micros: Some(42),
        input_tokens: Some(10),
        output_tokens: Some(5),
        ..Default::default()
    };
    let cost = super::pricing::resolve("openrouter", "unknown", &usage).await;
    assert_eq!(cost.micros, Some(42));
    assert!(cost.exact);
}

#[tokio::test]
async fn catalog_price_produces_an_estimate_only_with_real_tokens() {
    let usage = RequestUsage {
        input_tokens: Some(1_000),
        output_tokens: Some(500),
        ..Default::default()
    };
    let cost = super::pricing::resolve("openai", "gpt-4o", &usage).await;
    assert!(cost.micros.is_some_and(|value| value > 0));
    assert!(!cost.exact);

    let incomplete = RequestUsage {
        input_tokens: Some(1_000),
        ..Default::default()
    };
    assert_eq!(
        super::pricing::resolve("openai", "gpt-4o", &incomplete)
            .await
            .micros,
        None
    );
}

#[tokio::test]
async fn oauth_routes_never_inherit_public_api_prices() {
    let usage = RequestUsage {
        input_tokens: Some(1_000),
        output_tokens: Some(500),
        ..Default::default()
    };

    for connection_id in ["codex-oauth", "xai-oauth", "moonshot-oauth"] {
        assert_eq!(
            super::pricing::resolve(connection_id, "kimi-k2.7-code", &usage)
                .await
                .micros,
            None,
            "{connection_id}",
        );
    }
}

#[tokio::test]
async fn gpt_56_and_astra_never_use_an_unverified_catalog_price() {
    let usage = RequestUsage {
        input_tokens: Some(300_000),
        output_tokens: Some(10_000),
        cache_write_input_tokens: Some(20_000),
        ..Default::default()
    };

    assert_eq!(
        super::pricing::resolve("openai", "gpt-5.6-terra", &usage)
            .await
            .micros,
        None,
    );
    assert_eq!(
        super::pricing::resolve("openai", "gpt-6-astra", &usage)
            .await
            .micros,
        None,
    );
}

#[tokio::test]
async fn astra_loaded_catalog_cannot_override_unknown_or_reported_cost() {
    use crate::services::llm::{litellm_catalog, model_pricing};
    let entries = litellm_catalog::parse_catalog(
        r#"{
        "openai/gpt-6-astra": {"litellm_provider":"openai", "mode":"chat",
            "input_cost_per_token":0.00001, "output_cost_per_token":0.00005,
            "cache_creation_input_token_cost":0.0000125},
        "openrouter/openai/gpt-6-astra": {"litellm_provider":"openrouter", "mode":"chat",
            "input_cost_per_token":0.00001, "output_cost_per_token":0.00005,
            "cache_creation_input_token_cost":0.0000125}
    }"#,
    );
    let mut previous = Vec::with_capacity(2);
    {
        let mut catalog = litellm_catalog::get_lock().write().await;
        for (key, entry) in entries {
            // Preserve any existing capabilities; only pricing is under test.
            let old = catalog.get(&key).cloned();
            let mut replacement = old.clone().unwrap_or(entry.clone());
            replacement.input_cost_per_token = entry.input_cost_per_token;
            replacement.output_cost_per_token = entry.output_cost_per_token;
            replacement.cache_creation_input_token_cost = entry.cache_creation_input_token_cost;
            catalog.insert(key.clone(), replacement);
            previous.push((key, old));
        }
    }
    let usage = RequestUsage {
        input_tokens: Some(300_000),
        output_tokens: Some(100),
        cache_write_input_tokens: Some(20_000),
        ..Default::default()
    };
    let mut results = Vec::with_capacity(2);
    for (provider, model) in [
        ("openai", "gpt-6-astra"),
        ("openrouter", "openai/gpt-6-astra"),
    ] {
        let loaded = model_pricing::lookup(provider, model).await;
        let estimated = super::pricing::resolve(provider, model, &usage).await;
        let exact = super::pricing::resolve(
            provider,
            model,
            &RequestUsage {
                exact_cost_usd_micros: Some(1234),
                ..usage.clone()
            },
        )
        .await;
        results.push((loaded, estimated, exact));
    }
    {
        let mut catalog = litellm_catalog::get_lock().write().await;
        for (key, entry) in previous {
            if let Some(entry) = entry {
                catalog.insert(key, entry);
            } else {
                catalog.remove(&key);
            }
        }
    }
    for (loaded, estimated, exact) in results {
        assert_eq!(loaded.unwrap().input_cost_per_token, Some(0.00001));
        assert_eq!(estimated.micros, None);
        assert!(!estimated.exact);
        assert_eq!(exact.micros, Some(1234));
        assert!(exact.exact);
    }
}
