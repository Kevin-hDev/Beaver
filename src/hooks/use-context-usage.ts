import { useMemo } from "react";
import { finalizeContextUsage, type ContextUsageBreakdown } from "./context-usage-breakdown";
import { buildContextTokenBuckets, mergeContextTokenBuckets } from "./context-usage-buckets";
import { buildLiveContextMessage, type LiveContextState } from "./context-usage-live-message";
import { useContextHiddenUsage } from "./use-context-hidden-usage";
import type { AgentMessage } from "@/types/agent";

interface UseContextUsageArgs {
  sessionId: string;
  model: string;
  provider: string;
  messages: AgentMessage[];
  used?: number;
  stream: LiveContextState;
  workingDir?: string;
  permissionMode?: string;
  planMode?: boolean;
  supportsTools?: boolean;
}

export function useContextUsage({
  sessionId,
  model,
  provider,
  messages,
  used,
  stream,
  workingDir,
  permissionMode,
  planMode,
  supportsTools,
}: UseContextUsageArgs): ContextUsageBreakdown {
  const hiddenUsage = useContextHiddenUsage({
    sessionId,
    model,
    provider,
    workingDir,
    permissionMode,
    planMode,
    supportsTools,
  });
  const {
    completedSegments,
    currentContent,
    currentContentPhase,
    currentThinking,
    currentTools,
  } = stream;
  const persistedBuckets = useMemo(
    () => buildContextTokenBuckets(messages),
    [messages],
  );
  const hiddenBuckets = useMemo(
    () => buildContextTokenBuckets([], hiddenUsage),
    [hiddenUsage],
  );
  const liveMessage = useMemo(() => buildLiveContextMessage({
    completedSegments,
    currentContent,
    currentContentPhase,
    currentThinking,
    currentTools,
  }), [
    completedSegments,
    currentContent,
    currentContentPhase,
    currentThinking,
    currentTools,
  ]);
  const liveBuckets = useMemo(
    () => buildContextTokenBuckets(liveMessage ? [liveMessage] : []),
    [liveMessage],
  );

  return useMemo(
    () => finalizeContextUsage(mergeContextTokenBuckets(
      persistedBuckets,
      hiddenBuckets,
      liveBuckets,
    ), used),
    [persistedBuckets, hiddenBuckets, liveBuckets, used],
  );
}
