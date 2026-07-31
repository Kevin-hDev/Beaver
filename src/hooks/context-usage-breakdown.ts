import type { AgentMessage, ToolActivityRecord, ToolCallRequest } from "@/types/agent";
import { textUnits } from "./agent-token-estimate";
import { restoredToolArguments } from "./agent-chat-utils";

const CHARS_PER_TOKEN = 4;
const IMAGE_TOKEN_ESTIMATE = 1_100;

export const CONTEXT_USAGE_KEYS = [
  "messages",
  "systemTools",
  "mcpConnectors",
  "skills",
  "memory",
  "metaContext",
  "systemPrompt",
] as const;

type ContextUsageKey = typeof CONTEXT_USAGE_KEYS[number];

export interface ContextUsageItem {
  key: ContextUsageKey;
  tokens: number;
  percentage: number;
}

export interface ContextUsageBreakdown {
  used: number;
  items: ContextUsageItem[];
}

export interface ContextUsageOptions {
  observedUsed?: number;
  liveMessageTokens?: number;
  systemPromptTokens?: number;
  metaContextTokens?: number;
  skillContextTokens?: number;
  memoryContextTokens?: number;
  systemToolDefinitionTokens?: number;
  mcpDefinitionTokens?: number;
}

type TokenBuckets = Record<ContextUsageKey, number>;

export function buildContextUsageBreakdown(
  messages: AgentMessage[],
  options: ContextUsageOptions = {},
): ContextUsageBreakdown {
  const buckets: TokenBuckets = {
    messages: options.liveMessageTokens ?? 0,
    systemTools: options.systemToolDefinitionTokens ?? 0,
    mcpConnectors: options.mcpDefinitionTokens ?? 0,
    skills: options.skillContextTokens ?? 0,
    memory: options.memoryContextTokens ?? 0,
    metaContext: options.metaContextTokens ?? 0,
    systemPrompt: options.systemPromptTokens ?? 0,
  };

  for (const message of messages) {
    addMessageTokens(buckets, message);
  }

  const bucketTotal = sumBuckets(buckets);
  const observed = options.observedUsed ?? bucketTotal;
  const hasObservedTotal =
    options.observedUsed !== undefined && (observed > 0 || bucketTotal === 0);
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

function addMessageTokens(buckets: TokenBuckets, message: AgentMessage) {
  const baseUnits = textUnits(message.content)
    + (message.thinking ? textUnits(message.thinking) : 0)
    + fileUnits(message);
  buckets.messages += unitsToTokens(baseUnits);

  for (const call of message.tool_calls ?? []) {
    addToolCallTokens(buckets, call);
  }

  for (const activity of message.tool_activities ?? []) {
    addToolActivityTokens(buckets, activity);
  }

  if (message.role === "tool" && message.tool_name) {
    addNamedToolPayload(buckets, message.tool_name, message.content);
  }
}

function addToolCallTokens(buckets: TokenBuckets, call: ToolCallRequest) {
  const units = textUnits(call.function.name)
    + textUnits(JSON.stringify(call.function.arguments));
  buckets[toolBucket(call.function.name)] += unitsToTokens(units);
}

function addToolActivityTokens(buckets: TokenBuckets, activity: ToolActivityRecord) {
  const units = textUnits(activity.name)
    + textUnits(JSON.stringify(restoredToolArguments(activity)))
    + textUnits(activity.result ?? "");
  const bucket = activity.domain === "memory" ? "memory" : toolBucket(activity.name);
  buckets[bucket] += unitsToTokens(units);
}

function addNamedToolPayload(buckets: TokenBuckets, name: string, content: string) {
  buckets[toolBucket(name)] += unitsToTokens(textUnits(content));
}

function fileUnits(message: AgentMessage): number {
  let units = 0;
  for (const file of message.files ?? []) {
    units += textUnits(file.name);
    if (isImageFile(file.name) || file.thumbnail) {
      units += IMAGE_TOKEN_ESTIMATE * CHARS_PER_TOKEN;
    }
  }
  return units;
}

function toolBucket(name: string): "systemTools" | "mcpConnectors" | "metaContext" {
  if (name === "load_skill") return "metaContext";
  return isMcpTool(name) ? "mcpConnectors" : "systemTools";
}

function isMcpTool(name: string): boolean {
  return name === "mcp" || name === "mcp_tool" || name === "search_mcp_tools" || name.startsWith("mcp_");
}

function isImageFile(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return ["png", "jpg", "jpeg", "gif", "webp"].includes(ext);
}

function unitsToTokens(units: number): number {
  return Math.ceil(units / CHARS_PER_TOKEN);
}

function sumBuckets(buckets: TokenBuckets): number {
  return CONTEXT_USAGE_KEYS.reduce((sum, key) => sum + buckets[key], 0);
}

function fitBucketsToObserved(buckets: TokenBuckets, observed: number) {
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
    "messages",
    "systemTools",
    "mcpConnectors",
    "memory",
    "systemPrompt",
    "skills",
  ];
  for (const key of reductionOrder) {
    if (excess <= 0) break;
    const removed = Math.min(buckets[key], excess);
    buckets[key] -= removed;
    excess -= removed;
  }
}
