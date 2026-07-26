export type UsageAvailability = "complete" | "partial" | "unavailable";
export type CostQuality = "exact" | "estimated" | "partial" | "unavailable";
export type UsagePeriodId = "today" | "seven_days" | "thirty_days" | "all_time";

export interface UsageAggregate {
  tokens: {
    input_tokens: number;
    output_tokens: number;
    cached_input_tokens: number;
    reasoning_output_tokens: number;
    total_tokens: number;
  };
  request_count: number;
  usage_request_count: number;
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
  notice_code: string | null;
  refreshed_at: number;
  stale: boolean;
}
