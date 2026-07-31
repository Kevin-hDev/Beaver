import { useMemo } from "react";
import { buildContextUsageBreakdown, type ContextUsageBreakdown } from "./context-usage-breakdown";
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
  const liveMessage = useMemo(() => buildLiveContextMessage(stream), [stream]);

  return useMemo(
    () => buildContextUsageBreakdown(liveMessage ? [...messages, liveMessage] : messages, {
      ...hiddenUsage,
      observedUsed: used,
    }),
    [messages, liveMessage, hiddenUsage, used],
  );
}
