export type UsageAvailability = "complete" | "partial" | "unavailable";
export type CostQuality = "exact" | "estimated" | "partial" | "unavailable";
export type UsagePeriodId = "today" | "seven_days" | "thirty_days" | "all_time";

export interface UsageAggregate {
  tokens: {
    input_tokens: number;
    output_tokens: number;
    cached_input_tokens: number;
    cache_write_input_tokens: number;
    cache_miss_input_tokens: number;
    reasoning_output_tokens: number;
    total_tokens: number;
  };
  request_count: number;
  usage_request_count: number;
  cache_read_request_count: number;
  cache_write_request_count: number;
  cache_miss_request_count: number;
  cost_usd_micros: number;
  priced_request_count: number;
  exact_cost_request_count: number;
}

export interface UsagePeriod {
  period: UsagePeriodId;
  totals: UsageAggregate;
  origins: {
    manual_chat: UsageAggregate;
    external_channel: UsageAggregate;
    automation: UsageAggregate;
  };
  workloads: {
    primary: UsageAggregate;
    subagent: UsageAggregate;
    compression: UsageAggregate;
  };
  cost_quality: CostQuality;
}

export interface ProviderUsageSnapshot {
  connection_id: string;
  canonical_provider_id: string;
  auth_source: "api" | "oauth";
  availability: UsageAvailability;
  windows: Array<{
    label_code: string;
    group_code?: string | null;
    group_name?: string | null;
    used: number | null;
    limit: number | null;
    remaining: number | null;
    used_percent: number | null;
    resets_at: number | null;
  }>;
  balances: Array<{ label_code: string; amount: string; currency: string }>;
  local_periods: UsagePeriod[];
  request_metrics: {
    availability: "complete" | "empty" | "unavailable";
    recent: ProviderRequestMetric[];
    sessions: ProviderRequestSession[];
  };
  notice_code: string | null;
  refreshed_at: number;
  stale: boolean;
}

export interface ProviderRequestMetric {
  started_at_ms: number;
  connection_id: string;
  canonical_provider_id: string;
  api_format: "chat_completions" | "responses" | "gemini_native";
  model: string;
  routed_provider: string | null;
  routed_model: string | null;
  session_id: string | null;
  request_id: string;
  turn: number | null;
  attempt: number;
  workload: "primary" | "subagent" | "compression";
  origin: "manual_chat" | "external_channel" | "automation";
  status: "completed" | "interrupted" | "cancelled" | "failed";
  fast_requested: boolean;
  service_tier_served: "fast" | "default" | "unknown";
  timing: {
    headers_ms: number | null;
    first_event_ms: number | null;
    first_useful_ms: number | null;
    total_ms: number;
  };
  usage: {
    input_tokens: number | null;
    output_tokens: number | null;
    cached_input_tokens: number | null;
    cache_write_input_tokens: number | null;
    cache_miss_input_tokens: number | null;
    cache_miss_source: "unknown" | "reported" | "calculated";
    cache_status: "unknown" | "reported" | "invalid";
    reasoning_output_tokens: number | null;
    total_tokens: number | null;
    exact_cost_usd_micros: number | null;
  } | null;
  usage_complete: boolean;
}

export interface ProviderRequestSession {
  session_id: string;
  attempt_count: number;
  completed_count: number;
  usage_complete_count: number;
  cache_observation_count: number;
  cache_read_observation_count: number;
  cache_write_observation_count: number;
  cache_miss_observation_count: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
  cache_miss_tokens: number;
  total_duration_ms: number;
  latest_started_at_ms: number;
}
