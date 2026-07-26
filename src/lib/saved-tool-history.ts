import type { AgentMessage, SavedSegment, ToolActivityRecord, ToolCallRequest } from "@/types/agent";

const MAX_TOOLS_PER_SEGMENT = 64;
const MAX_TOOL_NAME_CHARS = 128;

export function normalizeSavedToolHistory(messages: AgentMessage[]): AgentMessage[] {
  const normalized: AgentMessage[] = [];
  let technicalGroup: AgentMessage[] = [];

  const flush = () => {
    if (technicalGroup.length === 0) return;
    normalized.push(...normalizeGroup(technicalGroup));
    technicalGroup = [];
  };

  for (const message of messages) {
    if (message.role === "user") {
      flush();
      normalized.push(message);
    } else {
      technicalGroup.push(message);
    }
  }
  flush();
  return normalized;
}

function normalizeGroup(group: AgentMessage[]): AgentMessage[] {
  if (group.some((message) => message.segments?.length)) return group;
  const hasTechnicalHistory = group.some((message) => (
    message.role === "tool" || !!message.tool_calls?.length || !!message.thinking
  ));
  if (!hasTechnicalHistory) return group;

  const segments = buildSegments(group);
  const anchor = findLast(group, (message) => message.role === "assistant");
  if (!anchor || segments.length === 0) return group;
  const tools = segments.flatMap((segment) => segment.tools);

  return [{
    ...anchor,
    content: segments.map((segment) => segment.content).filter(Boolean).join("\n\n"),
    thinking: segments.map((segment) => segment.thinking).filter(Boolean).join("\n\n") || undefined,
    tool_calls: undefined,
    tool_name: undefined,
    tool_activities: tools.length > 0 ? tools : undefined,
    segments,
  }];
}

function buildSegments(group: AgentMessage[]): SavedSegment[] {
  const segments: SavedSegment[] = [];
  const lastAssistantIndex = group.map((message) => message.role).lastIndexOf("assistant");
  for (let index = 0; index < group.length; index += 1) {
    const message = group[index];
    if (message.role !== "assistant") continue;
    const calls = (message.tool_calls ?? []).slice(0, MAX_TOOLS_PER_SEGMENT);
    if (calls.length > 0) {
      const tools = calls.map(toolRecord);
      let resultIndex = index + 1;
      while (resultIndex < group.length && group[resultIndex].role === "tool") {
        attachResult(tools, group[resultIndex]);
        resultIndex += 1;
      }
      segments.push({
        thinking: message.thinking,
        content: message.content,
        tools,
        phase: "work",
      });
      index = resultIndex - 1;
      continue;
    }

    const isLastAssistant = index === lastAssistantIndex;
    if (isLastAssistant && message.content) {
      if (message.thinking) {
        segments.push({ thinking: message.thinking, content: "", tools: [], phase: "work" });
      }
      segments.push({ content: message.content, tools: [], phase: "final" });
    } else if (message.content || message.thinking) {
      segments.push({
        thinking: message.thinking,
        content: message.content,
        tools: [],
        phase: "work",
      });
    }
  }
  return segments;
}

function toolRecord(call: ToolCallRequest): ToolActivityRecord {
  const name = typeof call.function.name === "string"
    ? call.function.name.slice(0, MAX_TOOL_NAME_CHARS)
    : "tool";
  const args = safeArgs(call.function.arguments);
  return {
    name,
    summary: toolSummary(name, args),
    args,
    content: name === "write_file" ? stringArg(args, "content") : undefined,
    old_text: name === "edit_file" ? stringArg(args, "old_string") : undefined,
    new_text: name === "edit_file" ? stringArg(args, "new_string") : undefined,
  };
}

function attachResult(tools: ToolActivityRecord[], message: AgentMessage) {
  const matching = tools.find((tool) => tool.result === undefined && tool.name === message.tool_name);
  const pending = matching ?? tools.find((tool) => tool.result === undefined);
  if (pending) pending.result = message.content;
}

function toolSummary(name: string, args: Record<string, unknown>): string {
  const key = name === "bash" ? "command"
    : name === "web_search" ? "query"
    : name === "web_fetch" ? "url"
    : name === "grep" || name === "glob" ? "pattern"
    : "path";
  const value = stringArg(args, key);
  return value || JSON.stringify(args).slice(0, 80);
}

function stringArg(args: Record<string, unknown>, key: string): string {
  return typeof args[key] === "string" ? args[key] : "";
}

function safeArgs(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function findLast<T>(items: T[], predicate: (item: T) => boolean): T | undefined {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    if (predicate(items[index])) return items[index];
  }
  return undefined;
}
