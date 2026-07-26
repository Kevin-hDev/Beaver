import type { AgentMessage, SubagentInfo, ToolActivityRecord } from "@/types/agent";

const MAX_MESSAGE_SUBAGENTS = 64;
const MAX_KNOWN_SUBAGENTS = 512;
const MAX_SEGMENTS_INSPECTED = 256;
const MAX_TOOLS_INSPECTED = 2048;
const MAX_PROMPT_PREVIEW = 120;
const MAX_ARG_SCAN_CHARS = 2048;
const MAX_RESULT_SCAN_CHARS = 512;
const SESSION_ID_RE = /^[A-Za-z0-9_-]{1,128}$/;
const SUBAGENT_ID_RE = /<subagent\b[^>]*\bid="([^"]{1,128})"/;

export function collectMessageSubagents(
  message: AgentMessage,
  knownSubagents: SubagentInfo[] = [],
): SubagentInfo[] {
  const known = knownSubagentsById(knownSubagents);
  const collected: SubagentInfo[] = [];
  const seen = new Set<string>();

  for (const tool of messageSubagentTools(message)) {
    if (collected.length >= MAX_MESSAGE_SUBAGENTS) break;
    const sessionId = extractSubagentId(tool);
    if (!sessionId || seen.has(sessionId)) continue;
    seen.add(sessionId);
    collected.push(mergeSubagentInfo(fallbackSubagentInfo(sessionId, tool), known.get(sessionId)));
  }

  return collected;
}

export function collectMessagesSubagents(
  messages: AgentMessage[],
  knownSubagents: SubagentInfo[] = [],
): SubagentInfo[] {
  const collected: SubagentInfo[] = [];
  const seen = new Set<string>();
  for (const message of messages.slice(0, MAX_SEGMENTS_INSPECTED)) {
    for (const subagent of collectMessageSubagents(message, knownSubagents)) {
      if (collected.length >= MAX_MESSAGE_SUBAGENTS) return collected;
      if (seen.has(subagent.sessionId)) continue;
      seen.add(subagent.sessionId);
      collected.push(subagent);
    }
  }
  return collected;
}

function* messageSubagentTools(message: AgentMessage): Generator<ToolActivityRecord> {
  let inspectedTools = 0;
  if (message.segments?.length) {
    let inspectedSegments = 0;
    for (const segment of message.segments) {
      if (inspectedSegments >= MAX_SEGMENTS_INSPECTED || inspectedTools >= MAX_TOOLS_INSPECTED) return;
      inspectedSegments += 1;
      if (segment.phase === "final") continue;
      for (const tool of segment.tools) {
        if (inspectedTools >= MAX_TOOLS_INSPECTED) return;
        inspectedTools += 1;
        if (tool.name === "delegate_task") yield tool;
      }
    }
    return;
  }
  for (const tool of message.tool_activities ?? []) {
    if (inspectedTools >= MAX_TOOLS_INSPECTED) return;
    inspectedTools += 1;
    if (tool.name === "delegate_task") yield tool;
  }
}

function knownSubagentsById(subagents: SubagentInfo[]): Map<string, SubagentInfo> {
  const byId = new Map<string, SubagentInfo>();
  for (const subagent of subagents) {
    if (byId.size >= MAX_KNOWN_SUBAGENTS) break;
    if (!isSessionId(subagent.sessionId)) continue;
    byId.set(subagent.sessionId, subagent);
  }
  return byId;
}

function extractSubagentId(tool: ToolActivityRecord): string | null {
  const fromResult = typeof tool.result === "string"
    ? SUBAGENT_ID_RE.exec(prefix(tool.result, MAX_RESULT_SCAN_CHARS))?.[1]
    : undefined;
  const candidate = fromResult || stringArg(tool.args, "subagent_id");
  return candidate && isSessionId(candidate) ? candidate : null;
}

function fallbackSubagentInfo(sessionId: string, tool: ToolActivityRecord): SubagentInfo {
  const type = stringArg(tool.args, "subagent_type") === "coder" ? "coder" : "explorer";
  return {
    sessionId,
    name: stringArg(tool.args, "display_name") || stringArg(tool.args, "name") || "agent",
    type,
    status: tool.is_error ? "failed" : "completed",
    promptPreview: stringArg(tool.args, "prompt", MAX_PROMPT_PREVIEW),
    description: stringArg(tool.args, "description") || undefined,
  };
}

function mergeSubagentInfo(fallback: SubagentInfo, known?: SubagentInfo): SubagentInfo {
  if (!known) return fallback;
  return {
    ...fallback,
    ...known,
    promptPreview: known.promptPreview || fallback.promptPreview,
    description: known.description || fallback.description,
  };
}

function stringArg(
  args: Record<string, unknown> | undefined,
  key: string,
  maxChars = MAX_ARG_SCAN_CHARS,
): string {
  const value = args?.[key];
  return typeof value === "string" ? limitChars(prefix(value, maxChars), maxChars).trim() : "";
}

function isSessionId(value: string): boolean {
  return SESSION_ID_RE.test(value);
}

function limitChars(value: string, maxChars: number): string {
  return Array.from(value).slice(0, maxChars).join("");
}

function prefix(value: string, maxChars: number): string {
  return value.length > maxChars ? value.slice(0, maxChars) : value;
}
