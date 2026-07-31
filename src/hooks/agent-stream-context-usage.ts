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
  const startsRequest = usage.estimated && usage.outputTokens === 0;
  if (!startsRequest) {
    const outputDelta = usage.outputTokens - state.contextOutputTokens;
    state.streamedMessageTokens = Math.max(
      0,
      state.streamedMessageTokens + outputDelta,
    );
  }
  state.contextInputTokens = usage.inputTokens;
  state.contextOutputTokens = usage.outputTokens;
  state.hasContextUsageSnapshot = true;
  state.sessionTokenCount = usage.contextTokens;
  state.sessionTokenCountEstimated = usage.estimated;
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
  const delta = cumulativeTokens - previousTokens;

  state.contextOutputTokens = cumulativeTokens;
  state.streamedMessageTokens += delta;
  state.liveTokenCount = cumulativeTokens;
  state.sessionTokenCount = state.contextInputTokens + cumulativeTokens;
  state.sessionTokenCountEstimated = true;
}
