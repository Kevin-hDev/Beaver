import type {
  CompressionBandSettings,
  CompressionLimitsView,
  CompressionProfile,
  CompressionProfilesView,
} from "@/types/compression-profile.generated";

function band(
  recentMessageCount: number,
  summaryMaxTokens: number,
  toolResultCount: number,
  recentFileCount: number,
  imageCount: number,
): CompressionBandSettings {
  return {
    recent_message_count: recentMessageCount,
    summary_max_tokens: summaryMaxTokens,
    tool_result_count: toolResultCount,
    recent_file_count: recentFileCount,
    image_count: imageCount,
    include_work_state: true,
  };
}

export function compressionLimitsFixture(): CompressionLimitsView {
  return {
    max_profiles: 20,
    max_profile_name_chars: 48,
    max_custom_prompt_chars: 32_000,
    max_messages: 8,
    max_tool_results: 50,
    max_files: 15,
    max_images: 16,
    min_summary_tokens: 1_000,
    max_summary_tokens: 8_000,
    min_threshold_percent: 1,
    max_threshold_percent: 90,
    under_64k_upper_exclusive: 64_000,
    compact_upper_exclusive: 128_000,
  };
}

export function compressionProfilesViewFixture(
  profiles = [compressionProfileFixture()],
  active = profiles[0]?.id ?? "beaver",
): CompressionProfilesView {
  return {
    automatic_enabled: true,
    global_profile_id: active,
    global_selection_revision: 1,
    profiles,
    limits: compressionLimitsFixture(),
  };
}

export function compressionProfileFixture(): CompressionProfile {
  return {
    id: "beaver",
    name: "Beaver",
    revision: 1,
    threshold_percent: 90,
    allow_under_64k: false,
    system_prompt: "System prompt",
    handoff_prompt: "Handoff prompt",
    under_64k: band(2, 2_000, 5, 3, 2),
    compact: band(4, 4_000, 10, 5, 4),
    large: band(4, 6_000, 10, 5, 4),
  };
}
