import { useMemo } from "react";
import { finalizeContextUsage, type ContextUsageBreakdown } from "./context-usage-breakdown";
import {
  buildContextTokenBuckets,
  mergeContextTokenBuckets,
  type ContextTokenBuckets,
} from "./context-usage-buckets";
import { buildLiveContextMessage, type LiveContextState } from "./context-usage-live-message";
import { resolvePreparedContextBuckets } from "./context-usage-stream";
import { useContextHiddenUsage } from "./use-context-hidden-usage";
import type { AgentMessage } from "@/types/agent";

interface UseContextUsageArgs {
  sessionId: string;
  model: string;
  provider: string;
  messages: AgentMessage[];
  stream: LiveContextState & {
    contextUsageBuckets: ContextTokenBuckets | null;
    contextUsageBaseSegments: number;
    contextUsageIncludesReasoning: boolean;
  };
  workingDir?: string;
  permissionMode?: string;
  planMode?: boolean;
  supportsTools?: boolean;
  contextUsageIncludesReasoning?: boolean;
}

export function useContextUsage({
  sessionId,
  model,
  provider,
  messages,
  stream,
  workingDir,
  permissionMode,
  planMode,
  supportsTools,
  contextUsageIncludesReasoning: modelIncludesReasoning,
}: UseContextUsageArgs): ContextUsageBreakdown {
  const hiddenUsage = useContextHiddenUsage({
    enabled: !stream.contextUsageBuckets,
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
    contextUsageBuckets,
    contextUsageBaseSegments,
    contextUsageIncludesReasoning,
  } = stream;
  const includeThinking = modelIncludesReasoning ?? contextUsageIncludesReasoning;
  const preparedBuckets = useMemo(
    () => resolvePreparedContextBuckets({
      completedSegments,
      currentContent,
      currentContentPhase,
      currentThinking,
      currentTools,
      contextUsageBuckets,
      contextUsageBaseSegments,
      contextUsageIncludesReasoning,
    }),
    [
      contextUsageBuckets,
      contextUsageBaseSegments,
      contextUsageIncludesReasoning,
      completedSegments,
      currentContent,
      currentContentPhase,
      currentThinking,
      currentTools,
    ],
  );
  const persistedBuckets = useMemo(
    () => buildContextTokenBuckets(messages, { includeThinking }),
    [messages, includeThinking],
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
    () => buildContextTokenBuckets(liveMessage ? [liveMessage] : [], { includeThinking }),
    [liveMessage, includeThinking],
  );

  return useMemo(
    () => finalizeContextUsage(preparedBuckets ?? mergeContextTokenBuckets(
      persistedBuckets, hiddenBuckets, liveBuckets,
    )),
    [preparedBuckets, persistedBuckets, hiddenBuckets, liveBuckets],
  );
}
