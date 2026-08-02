import type { AgentMessage, AgentSession } from "@/types/agent";
import { restoredToolArguments } from "./agent-chat-utils";
import { toolsFromMessage } from "@/lib/message-tools";

const CHARS_PER_TOKEN = 4;

export function estimateAgentMessagesTokens(messages: AgentMessage[]): number {
  return messages.reduce((sum, message) => sum + estimateMessage(message), 0);
}

export function resolveSessionContext(session: AgentSession): {
  sessionTokenCount: number;
  hasContextUsageSnapshot: boolean;
  contextUsageVisible: boolean;
} {
  const contextTokens = session.context_tokens ?? 0;
  return {
    sessionTokenCount: contextTokens || session.accumulated_tokens
      || estimateAgentMessagesTokens(session.messages),
    hasContextUsageSnapshot: contextTokens > 0,
    contextUsageVisible: session.messages.some((message) => message.role === "assistant"),
  };
}

function estimateMessage(message: AgentMessage): number {
  let units = textUnits(message.content);
  units += message.thinking ? textUnits(message.thinking) : 0;
  if (message.tool_calls) {
    for (const call of message.tool_calls) {
      units += textUnits(call.function.name);
      units += textUnits(JSON.stringify(call.function.arguments));
    }
  }
  for (const activity of toolsFromMessage(message)) {
    units += textUnits(activity.name);
    units += textUnits(JSON.stringify(restoredToolArguments(activity)));
    units += activity.result ? textUnits(activity.result) : 0;
  }
  return Math.ceil(units / CHARS_PER_TOKEN);
}

export function textUnits(input: string): number {
  let units = 0;
  for (const char of input) units += charUnits(char);
  return units;
}

function charUnits(char: string): number {
  const cp = char.codePointAt(0) ?? 0;
  if (cp <= 0x7f) return 1;
  if (isWideOrEmoji(cp)) return 5;
  return 2;
}

function isWideOrEmoji(cp: number): boolean {
  return (
    (cp >= 0x1100 && cp <= 0x11ff) ||
    (cp >= 0x2e80 && cp <= 0x2eff) ||
    (cp >= 0x2f00 && cp <= 0x2fdf) ||
    (cp >= 0x3000 && cp <= 0x30ff) ||
    (cp >= 0x3130 && cp <= 0x318f) ||
    (cp >= 0x31a0 && cp <= 0x31bf) ||
    (cp >= 0x31f0 && cp <= 0x31ff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0xac00 && cp <= 0xd7af) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0xfe00 && cp <= 0xfe0f) ||
    (cp >= 0xff00 && cp <= 0xffef) ||
    (cp >= 0x1f000 && cp <= 0x1faff) ||
    (cp >= 0x20000 && cp <= 0x2ceaf)
  );
}
