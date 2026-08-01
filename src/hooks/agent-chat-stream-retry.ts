import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { RetryIndicatorState } from "@/types/agent";

const PROVIDER_RETRY_REASON = "agentLocal.retry.provider";

export function applyRetryIndicator(
  state: ManagedStreamState,
  indicator: RetryIndicatorState,
  now: number,
) {
  state.retryIndicator = indicator;
  if (indicator.reasonKey !== PROVIDER_RETRY_REASON) return;

  const discardedTokens = safeTokenCount(state.contextOutputTokens);
  if (state.hasContextUsageSnapshot) {
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
  state.contextOutputTokens = 0;
  state.liveTokenCount = Math.max(
    0,
    safeTokenCount(state.liveTokenCount) - discardedTokens,
  );
  state.sessionTokenCount = safeTokenCount(state.contextInputTokens);
  state.segmentStartedAt = now;
}

function safeTokenCount(value: number): number {
  if (!Number.isFinite(value) || value <= 0) return 0;
  return Math.floor(value);
}
