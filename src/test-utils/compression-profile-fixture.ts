import type {
  CompressionBandSettings,
  CompressionProfile,
  TokenBudget,
} from "@/types/compression-profile.generated";

const fixed = (tokens: number): TokenBudget => ({
  mode: "fixed",
  fixed_tokens: tokens,
  percent_basis_points: 0,
  minimum_tokens: 0,
});

function band(scale = 1): CompressionBandSettings {
  const item = {
    enabled: true,
    max_items: Math.round(10 * scale),
    tokens_per_item: Math.round(500 * scale),
    total_tokens: Math.round(5_000 * scale),
  };
  const category = { enabled: true, tokens: fixed(Math.round(5_000 * scale)) };
  return {
    target_percent: 75,
    response_reserve: fixed(Math.round(4_000 * scale)),
    minimum_reduction: { mode: "minimum", fixed_tokens: 10_000, percent_basis_points: 2_500, minimum_tokens: 0 },
    summary_output: {
      window_limit: fixed(Math.round(8_000 * scale)),
      input_ratio_divisor: 3,
      input_floor_tokens: 1_000,
      input_ceiling_tokens: 16_000,
    },
    user_messages: category,
    assistant_messages: category,
    evidence_envelope: fixed(Math.round(10_000 * scale)),
    tools: item,
    files: item,
    modified_files: item,
    text_attachments: item,
    images: { enabled: true, max_items: 4, max_total_bytes: 16 * 1024 * 1024 },
    git_tokens: category,
    plan_and_tasks_tokens: category,
    subagent_detail_tokens: category,
    unresolved_state_tokens: category,
    critical_references: item,
  };
}
export function compressionProfileFixture(): CompressionProfile {
  return {
    id: "beaver",
    name: "Beaver",
    revision: 1,
    threshold_percent: 90,
    allow_under_64k: false,
    context_capacity_policy: "reduce_optional_categories",
    summary: {
      enabled: true,
      system_prompt: "System prompt",
      handoff_prompt: "Handoff prompt",
      model: { kind: "current" },
      fallback_model: null,
      ordinary_retries: 1,
      input_budget: fixed(60_000),
      failure_policy: "keep_history",
    },
    under_64k: band(0.5),
    compact: band(1),
    large: band(2),
    reduction_order: ["images", "files", "tools", "assistant_messages", "user_messages"],
  };
}
