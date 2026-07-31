import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { StreamEvent } from "@/types/agent";

type ContextUsageData = Extract<
  StreamEvent,
  { event: "contextUsage" }
>["data"];

export function applyContextUsage(
  state: ManagedStreamState,
  usage: ContextUsageData,
) {
  state.contextInputTokens = usage.inputTokens;
  state.contextOutputTokens = usage.outputTokens;
  state.contextLimitTokens = usage.contextLimit;
  state.hasContextUsageSnapshot = true;
  state.sessionTokenCount = usage.contextTokens;
  state.liveTokenCount = usage.outputTokens;
}

export function applyGeneratedTokenCount(
  state: ManagedStreamState,
  reportedTokens: number | undefined,
) {
  const previousTokens = Math.max(
    state.contextOutputTokens,
    state.liveTokenCount,
  );
  const nextRequestTokens = reportedTokens && reportedTokens > 0
    ? reportedTokens
    : previousTokens + 1;
  const cumulativeTokens = nextRequestTokens >= previousTokens
    ? nextRequestTokens
    : previousTokens + nextRequestTokens;

  state.contextOutputTokens = cumulativeTokens;
  state.liveTokenCount = cumulativeTokens;
  state.sessionTokenCount = state.contextInputTokens + cumulativeTokens;
}
