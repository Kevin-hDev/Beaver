import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { ContextTokenBuckets } from "./context-usage-buckets";
import type { StreamEvent } from "@/types/agent";
import type { RequestContextUsage } from "@/types/agent-session.generated";
import { resolveContextUsage } from "./agent-token-estimate";

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
  const resolved = resolveContextUsage(usage.record);
  const inputTokens = boundedTokens(resolved.used ?? 0);
  const outputTokens = boundedTokens(resolved.output ?? 0);
  const startsRequest = preparation?.state === "ready" || preparation?.state === "in_flight";
  if (startsRequest) {
    state.requestOutputTokens = 0;
  } else {
    state.liveTokenCount = adjustedTokens(
      state.liveTokenCount,
      outputTokens - state.requestOutputTokens,
    );
    state.requestOutputTokens = outputTokens;
  }
  state.contextUsageRecord = usage.record;
  state.contextLimitTokens = boundedTokens(resolved.max ?? 0);
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
  const previousTokens = boundedTokens(state.requestOutputTokens);
  const reported = reportedTokens === undefined ? 0 : boundedTokens(reportedTokens);
  const nextRequestTokens = reported > 0
    ? Math.max(previousTokens, reported)
    : boundedSum(previousTokens, 1);
  state.requestOutputTokens = nextRequestTokens;
  state.liveTokenCount = boundedSum(state.liveTokenCount, nextRequestTokens - previousTokens);
  const inputTokens = boundedTokens(resolveContextUsage(state.contextUsageRecord).used ?? 0);
  state.sessionTokenCount = boundedSum(
    inputTokens,
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
