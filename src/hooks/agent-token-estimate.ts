import type { AgentMessage, AgentSession } from "@/types/agent";
import type {
  ContextCountCoverage,
  ContextCountSource,
  ContextPreparationState,
  ContextRequestIdentity,
  ContextUsageRecord,
  RequestContextUsage,
} from "@/types/agent-session.generated";
import { restoredToolArguments } from "./agent-chat-utils";
import { toolsFromMessage } from "@/lib/message-tools";

const CHARS_PER_TOKEN = 4;

export const EMPTY_CONTEXT_USAGE_RECORD: ContextUsageRecord = {
  activeRequestId: null,
  currentPreparation: null,
  lastMeasurement: null,
  lastOutput: null,
};

export type ContextUsageStatus = "prepared" | "streaming" | "measured"
  | "completedEstimated" | "stale" | "interrupted" | "failed"
  | "reconstructed" | "partial" | "unavailable";

export interface ResolvedContextUsage {
  used: number | null;
  max: number | null;
  output: number | null;
  status: ContextUsageStatus;
  secondaryStatus: ContextUsageStatus | null;
  source: ContextCountSource | null;
  coverage: ContextCountCoverage | null;
  breakdown: RequestContextUsage | null;
}

export function estimateAgentMessagesTokens(messages: AgentMessage[]): number {
  return messages.reduce((sum, message) => sum + estimateMessage(message), 0);
}

export function resolveSessionContext(session: AgentSession): {
  sessionTokenCount: number;
  contextUsageRecord: ContextUsageRecord;
  contextUsageVisible: boolean;
} {
  const contextTokens = session.context_tokens ?? 0;
  return {
    sessionTokenCount: contextTokens || session.accumulated_tokens
      || estimateAgentMessagesTokens(session.messages),
    contextUsageRecord: session.context_usage ?? EMPTY_CONTEXT_USAGE_RECORD,
    contextUsageVisible: session.messages.some((message) => message.role === "assistant"),
  };
}

export function resolveContextUsage(
  record: ContextUsageRecord,
  reconstructedTokens = 0,
  reconstructedLimit = 0,
): ResolvedContextUsage {
  const preparation = record.currentPreparation;
  const active = preparation?.state === "ready" || preparation?.state === "in_flight"
    ? preparation
    : null;
  const primary = active ?? record.lastMeasurement
    ?? (preparation?.state === "completed" ? preparation : null);
  const input = primary?.input;
  const used = validCount(input?.tokens);
  const reconstructed = used === null ? validCount(reconstructedTokens) : null;
  const secondaryStatus = !active && record.lastMeasurement && preparation
    && (preparation.state !== "completed"
      || !sameIdentity(preparation.identity, record.lastMeasurement.identity))
    ? stateStatus(preparation.state)
    : null;
  return {
    used: used ?? reconstructed,
    max: primary
      ? validCount(primary.contextLimit)
      : reconstructed !== null ? validCount(reconstructedLimit) : null,
    output: validCount(record.lastOutput?.output.tokens),
    status: used !== null
      ? input?.coverage === "partial" ? "partial" : primaryStatus(active, record, preparation)
      : reconstructed !== null ? "reconstructed" : "unavailable",
    secondaryStatus,
    source: used !== null ? input?.source ?? null : reconstructed !== null ? "reconstructed" : null,
    coverage: used !== null ? input?.coverage ?? null : reconstructed !== null ? "complete" : null,
    breakdown: preparation?.breakdown ?? null,
  };
}

function primaryStatus(
  active: ContextUsageRecord["currentPreparation"],
  record: ContextUsageRecord,
  preparation: ContextUsageRecord["currentPreparation"],
): ContextUsageStatus {
  if (active?.state === "ready") return "prepared";
  if (active?.state === "in_flight") return "streaming";
  if (record.lastMeasurement) return "measured";
  return preparation?.state === "completed" ? "completedEstimated" : "unavailable";
}

function stateStatus(state: ContextPreparationState): ContextUsageStatus | null {
  if (state === "completed") return "completedEstimated";
  if (state === "stale" || state === "interrupted" || state === "failed") return state;
  return null;
}

function validCount(value: number | null | undefined): number | null {
  return typeof value === "number" && Number.isFinite(value) && value > 0
    ? Math.floor(value)
    : null;
}

function sameIdentity(
  left: ContextRequestIdentity,
  right: ContextRequestIdentity,
): boolean {
  return left.requestId === right.requestId
    && left.turnId === right.turnId
    && left.turn === right.turn
    && left.attempt === right.attempt;
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
