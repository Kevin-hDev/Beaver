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

export type ContextUsageOptions = ContextBucketOptions;

export function buildContextUsageBreakdown(
  messages: AgentMessage[],
  options: ContextUsageOptions = {},
): ContextUsageBreakdown {
  const buckets = buildContextTokenBuckets(messages, options);
  return finalizeContextUsage(buckets);
}

export function finalizeContextUsage(
  source: ContextTokenBuckets,
): ContextUsageBreakdown {
  const buckets = { ...source };
  const used = sumBuckets(buckets);
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
