import type { AgentSession } from "@/types/agent";
import { isKnownAgentErrorCode } from "./agent-error-codes";

export interface TerminalFailure {
  code: string;
  isConnection: boolean;
}

export function latestTerminalFailure(session: AgentSession): TerminalFailure | null {
  // Une exécution active a autorité sur les anciens diagnostics terminaux.
  if ((session.diagnostic_runs ?? []).some((run) => run.status === "running")) return null;
  const terminalRuns = (session.diagnostic_runs ?? [])
    .sort((left, right) => timestamp(right.ended_at ?? right.updated_at)
      - timestamp(left.ended_at ?? left.updated_at));
  const latest = terminalRuns[0];
  if (!latest || latest.status !== "failed") return null;

  const endedAt = timestamp(latest.ended_at ?? latest.updated_at);
  const hasNewerAssistant = session.messages.some((message) =>
    message.role === "assistant" && timestamp(message.timestamp) > endedAt);
  if (hasNewerAssistant) return null;

  const failure = (session.stream_failures ?? [])
    .filter((entry) => timestamp(entry.occurred_at) <= endedAt + 1_000)
    .sort((left, right) => timestamp(right.occurred_at) - timestamp(left.occurred_at))[0];
  const code = failure?.code && isKnownAgentErrorCode(failure.code)
    ? failure.code
    : "stream_interrupted";
  return {
    code,
    isConnection: failure?.is_connection ?? latest.error_type === "connection_lost",
  };
}

function timestamp(value: string | undefined): number {
  const parsed = value ? Date.parse(value) : Number.NaN;
  return Number.isFinite(parsed) ? parsed : 0;
}
