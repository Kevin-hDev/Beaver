import { useMemo } from "react";
import { buildContextUsageBreakdown, type ContextUsageBreakdown } from "./context-usage-breakdown";
import { useContextHiddenUsage } from "./use-context-hidden-usage";
import type { AgentMessage } from "@/types/agent";

interface UseContextUsageArgs {
  sessionId: string;
  model: string;
  provider: string;
  messages: AgentMessage[];
  used: number;
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

  return useMemo(
    () => buildContextUsageBreakdown(messages, { ...hiddenUsage, observedUsed: used }),
    [messages, hiddenUsage, used],
  );
}
