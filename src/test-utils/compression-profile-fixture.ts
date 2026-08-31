import type {
  CompressionBandSettings,
  CompressionProfile,
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
