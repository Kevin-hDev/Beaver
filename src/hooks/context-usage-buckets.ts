import type { AgentMessage, ToolActivityRecord, ToolCallRequest } from "@/types/agent";
import { restoredToolArguments } from "./agent-chat-utils";
import { textUnits } from "./agent-token-estimate";

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

export type ContextUsageKey = typeof CONTEXT_USAGE_KEYS[number];
export type ContextTokenBuckets = Record<ContextUsageKey, number>;

export interface ContextBucketOptions {
  includeThinking?: boolean;
  systemPromptTokens?: number;
  metaContextTokens?: number;
  skillContextTokens?: number;
  memoryContextTokens?: number;
  systemToolDefinitionTokens?: number;
  mcpDefinitionTokens?: number;
}

export function buildContextTokenBuckets(
  messages: AgentMessage[],
  options: ContextBucketOptions = {},
): ContextTokenBuckets {
  const buckets = emptyBuckets(options);
  const includeThinking = options.includeThinking ?? true;
  for (const message of messages) addMessageTokens(buckets, message, includeThinking);
  return buckets;
}

export function mergeContextTokenBuckets(
  ...sources: ContextTokenBuckets[]
): ContextTokenBuckets {
  const merged = emptyBuckets();
  for (const source of sources) {
    for (const key of CONTEXT_USAGE_KEYS) merged[key] += source[key];
  }
  return merged;
}

function emptyBuckets(options: ContextBucketOptions = {}): ContextTokenBuckets {
  return {
    messages: 0,
    systemTools: options.systemToolDefinitionTokens ?? 0,
    mcpConnectors: options.mcpDefinitionTokens ?? 0,
    skills: options.skillContextTokens ?? 0,
    memory: options.memoryContextTokens ?? 0,
    metaContext: options.metaContextTokens ?? 0,
    systemPrompt: options.systemPromptTokens ?? 0,
  };
}

function addMessageTokens(
  buckets: ContextTokenBuckets,
  message: AgentMessage,
  includeThinking: boolean,
) {
  const namedTool = message.role === "tool" && !!message.tool_name;
  const baseUnits = (namedTool ? 0 : textUnits(message.content))
    + (includeThinking && message.thinking ? textUnits(message.thinking) : 0)
    + fileUnits(message);
  buckets.messages += unitsToTokens(baseUnits);

  for (const call of message.tool_calls ?? []) addToolCallTokens(buckets, call);
  for (const activity of message.tool_activities ?? []) {
    addToolActivityTokens(buckets, activity);
  }
  if (namedTool && message.tool_name) {
    addNamedToolPayload(buckets, message.tool_name, message.content);
  }
}

function addToolCallTokens(buckets: ContextTokenBuckets, call: ToolCallRequest) {
  const units = textUnits(call.function.name)
    + textUnits(JSON.stringify(call.function.arguments));
  buckets[toolBucket(call.function.name)] += unitsToTokens(units);
}

function addToolActivityTokens(
  buckets: ContextTokenBuckets,
  activity: ToolActivityRecord,
) {
  const units = textUnits(activity.name)
    + textUnits(JSON.stringify(restoredToolArguments(activity)))
    + textUnits(activity.result ?? "");
  const bucket = activity.domain === "memory" ? "memory" : toolBucket(activity.name);
  buckets[bucket] += unitsToTokens(units);
}

function addNamedToolPayload(
  buckets: ContextTokenBuckets,
  name: string,
  content: string,
) {
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
  return name === "mcp" || name === "mcp_tool"
    || name === "search_mcp_tools" || name.startsWith("mcp_");
}

function isImageFile(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return ["png", "jpg", "jpeg", "gif", "webp"].includes(ext);
}

function unitsToTokens(units: number): number {
  return Math.ceil(units / CHARS_PER_TOKEN);
}
