import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { RetryIndicatorState } from "@/types/agent";
import { resolveContextUsage } from "./agent-token-estimate";

const PROVIDER_RETRY_REASON = "agentLocal.retry.provider";

export function applyRetryIndicator(
  state: ManagedStreamState,
  indicator: RetryIndicatorState,
  now: number,
) {
  state.retryIndicator = indicator;
  if (indicator.reasonKey !== PROVIDER_RETRY_REASON) return;

  const resolved = resolveContextUsage(state.contextUsageRecord);
  const discardedTokens = safeTokenCount(state.requestOutputTokens);
  if (resolved.used !== null) {
    const completedBeforeRequest = Math.min(
      safeTokenCount(state.contextUsageBaseSegments),
      state.completedSegments.length,
    );
    state.completedSegments = state.completedSegments.slice(0, completedBeforeRequest);
  }
  state.currentContent = "";
  state.currentContentPhase = undefined;
  state.currentThinking = "";
  state.currentTools = [];
  state.activeStreamItem = null;
  state.tps = 0;
  state.tpsEstimated = false;
  state.liveTokenCount = Math.max(
    0,
    safeTokenCount(state.liveTokenCount) - discardedTokens,
  );
  state.requestOutputTokens = 0;
  state.sessionTokenCount = safeTokenCount(resolved.used ?? 0);
  state.segmentStartedAt = now;
}

function safeTokenCount(value: number): number {
  if (!Number.isFinite(value) || value <= 0) return 0;
  return Math.floor(value);
}
