import type { AgentMessage } from "@/types/agent";
import {
  buildContextTokenBuckets,
  CONTEXT_USAGE_KEYS,
  type ContextBucketOptions,
  type ContextTokenBuckets,
  type ContextUsageKey,
} from "./context-usage-buckets";

export { CONTEXT_USAGE_KEYS } from "./context-usage-buckets";

export interface ContextUsageItem {
  key: ContextUsageKey;
  tokens: number;
  percentage: number;
}

export interface ContextUsageBreakdown {
  used: number;
  items: ContextUsageItem[];
}

export interface ContextUsageOptions extends ContextBucketOptions {
  observedUsed?: number;
}

export function buildContextUsageBreakdown(
  messages: AgentMessage[],
  options: ContextUsageOptions = {},
): ContextUsageBreakdown {
  const buckets = buildContextTokenBuckets(messages, options);
  return finalizeContextUsage(buckets, options.observedUsed);
}

export function finalizeContextUsage(
  source: ContextTokenBuckets,
  observedUsed?: number,
): ContextUsageBreakdown {
  const buckets = { ...source };
  const bucketTotal = sumBuckets(buckets);
  const observed = observedUsed ?? bucketTotal;
  const hasObservedTotal =
    observedUsed !== undefined && (observed > 0 || bucketTotal === 0);
  if (hasObservedTotal) {
    fitBucketsToObserved(buckets, observed);
  }

  const used = hasObservedTotal ? observed : bucketTotal;
  return {
    used,
    items: CONTEXT_USAGE_KEYS.map((key) => ({
      key,
      tokens: buckets[key],
      percentage: used > 0 ? (buckets[key] / used) * 100 : 0,
    })),
  };
}

function sumBuckets(buckets: ContextTokenBuckets): number {
  return CONTEXT_USAGE_KEYS.reduce((sum, key) => sum + buckets[key], 0);
}

function fitBucketsToObserved(buckets: ContextTokenBuckets, observed: number) {
  const total = sumBuckets(buckets);
  if (total <= observed) {
    buckets.metaContext += observed - total;
    return;
  }
  if (observed <= 0) {
    for (const key of CONTEXT_USAGE_KEYS) buckets[key] = 0;
    return;
  }

  let excess = total - observed;
  const reductionOrder: ContextUsageKey[] = [
    "metaContext",
    "systemTools",
    "mcpConnectors",
    "memory",
    "systemPrompt",
    "skills",
    "messages",
  ];
  for (const key of reductionOrder) {
    if (excess <= 0) break;
    const removed = Math.min(buckets[key], excess);
    buckets[key] -= removed;
    excess -= removed;
  }
}
