import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { ContextTokenBuckets } from "./context-usage-buckets";
import type { StreamEvent } from "@/types/agent";
import type { RequestContextUsage } from "@/types/agent-session.generated";

type ContextUsageData = Extract<
  StreamEvent,
  { event: "contextUsage" }
>["data"];

const MAX_TOKEN_COUNT = 0xffff_ffff;

export function applyContextUsage(
  state: ManagedStreamState,
  usage: ContextUsageData,
) {
  const preparation = usage.record.currentPreparation;
  const activePreparation = preparation && (
    preparation.state === "ready" || preparation.state === "in_flight"
  ) ? preparation : null;
  const selectedInput = activePreparation
    ?? usage.record.lastMeasurement
    ?? preparation;
  const inputTokens = boundedTokens(selectedInput?.input.tokens ?? 0);
  const outputTokens = boundedTokens(usage.record.lastOutput?.output.tokens ?? 0);
  const startsRequest = activePreparation !== null;
  if (!startsRequest) {
    state.liveTokenCount = adjustedTokens(
      state.liveTokenCount,
      outputTokens - state.contextOutputTokens,
    );
  }
  state.contextInputTokens = inputTokens;
  state.contextOutputTokens = outputTokens;
  state.contextLimitTokens = boundedTokens(selectedInput?.contextLimit ?? 0);
  state.hasContextUsageSnapshot = selectedInput !== null;
  state.sessionTokenCount = boundedSum(inputTokens, outputTokens);
  if (preparation?.breakdown) {
    state.contextUsageBuckets = boundedBuckets(preparation.breakdown);
    state.contextUsageBaseSegments = state.completedSegments.length;
    state.contextUsageIncludesReasoning = preparation.breakdown.reasoningIncluded === true;
  }
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

function boundedBuckets(
  source: RequestContextUsage,
): ContextTokenBuckets {
  return {
    messages: boundedTokens(source.messages),
    systemTools: boundedTokens(source.systemTools),
    mcpConnectors: boundedTokens(source.mcpConnectors),
    skills: boundedTokens(source.skills),
    memory: boundedTokens(source.memory),
    metaContext: boundedTokens(source.metaContext),
    systemPrompt: boundedTokens(source.systemPrompt),
  };
}
