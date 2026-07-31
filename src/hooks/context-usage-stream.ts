import { buildContextTokenBuckets, mergeContextTokenBuckets } from "./context-usage-buckets";
import { buildLiveContextMessage, type LiveContextState } from "./context-usage-live-message";
import type { ContextTokenBuckets } from "./context-usage-buckets";
import type { AgentMessage } from "@/types/agent";

export interface PreparedContextState extends LiveContextState {
  contextUsageBuckets: ContextTokenBuckets | null;
  contextUsageBaseSegments: number;
  contextUsageIncludesReasoning: boolean;
}

export function resolvePreparedContextBuckets(
  state: PreparedContextState,
  extraMessages: AgentMessage[] = [],
): ContextTokenBuckets | null {
  if (!state.contextUsageBuckets) return null;
  const firstPendingSegment = Math.min(
    Math.max(Math.floor(state.contextUsageBaseSegments), 0),
    state.completedSegments.length,
  );
  const liveMessage = buildLiveContextMessage({
    completedSegments: state.completedSegments.slice(firstPendingSegment),
    currentContent: state.currentContent,
    currentContentPhase: state.currentContentPhase,
    currentThinking: state.currentThinking,
    currentTools: state.currentTools,
  });
  const pending = liveMessage ? [liveMessage, ...extraMessages] : extraMessages;
  return mergeContextTokenBuckets(
    state.contextUsageBuckets,
    buildContextTokenBuckets(pending, {
      includeThinking: state.contextUsageIncludesReasoning,
    }),
  );
}
