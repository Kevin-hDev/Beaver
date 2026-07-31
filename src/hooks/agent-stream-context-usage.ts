import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { StreamEvent } from "@/types/agent";

type ContextUsageData = Extract<
  StreamEvent,
  { event: "contextUsage" }
>["data"];

const MAX_TOKEN_COUNT = 0xffff_ffff;

export function applyContextUsage(
  state: ManagedStreamState,
  usage: ContextUsageData,
) {
  const inputTokens = boundedTokens(usage.inputTokens);
  const outputTokens = boundedTokens(usage.outputTokens);
  const startsRequest = usage.estimated && outputTokens === 0;
  if (!startsRequest) {
    state.liveTokenCount = adjustedTokens(
      state.liveTokenCount,
      outputTokens - state.contextOutputTokens,
    );
  }
  state.contextInputTokens = inputTokens;
  state.contextOutputTokens = outputTokens;
  state.contextLimitTokens = boundedTokens(usage.contextLimit);
  state.hasContextUsageSnapshot = true;
  state.sessionTokenCount = boundedSum(inputTokens, outputTokens);
}

export function applyGeneratedTokenCount(
  state: ManagedStreamState,
  reportedTokens: number | undefined,
) {
  const previousTokens = boundedTokens(state.contextOutputTokens);
  const reported = reportedTokens === undefined ? 0 : boundedTokens(reportedTokens);
  const nextRequestTokens = reported > 0
    ? Math.max(previousTokens, reported)
    : boundedSum(previousTokens, 1);
  const delta = nextRequestTokens - previousTokens;

  state.contextOutputTokens = nextRequestTokens;
  state.liveTokenCount = boundedSum(state.liveTokenCount, delta);
  state.sessionTokenCount = boundedSum(
    state.contextInputTokens,
    nextRequestTokens,
  );
}

function boundedTokens(value: number): number {
  if (!Number.isFinite(value) || value <= 0) return 0;
  return Math.min(Math.floor(value), MAX_TOKEN_COUNT);
}

function boundedSum(left: number, right: number): number {
  return Math.min(boundedTokens(left) + boundedTokens(right), MAX_TOKEN_COUNT);
}

function adjustedTokens(value: number, delta: number): number {
  if (!Number.isFinite(delta)) return boundedTokens(value);
  return Math.max(0, Math.min(boundedTokens(value) + delta, MAX_TOKEN_COUNT));
}
