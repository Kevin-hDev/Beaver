import type { AgentSession } from "@/types/agent";

const KNOWN_CODES = new Set([
  "rate_limit",
  "auth_failed",
  "oauth_reauthentication_required",
  "provider_access_unavailable",
  "provider_connection_failed",
  "provider_temporarily_unavailable",
  "provider_request_rejected",
  "provider_payload_too_large",
  "provider_configuration_invalid",
  "provider_quota_exhausted",
]);

export interface TerminalFailure {
  code: string;
  isConnection: boolean;
  diagnosticSummary?: string;
}

export function latestTerminalFailure(session: AgentSession): TerminalFailure | null {
  const terminalRuns = (session.diagnostic_runs ?? [])
    .filter((run) => run.status !== "running")
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
  const code = failure?.code && KNOWN_CODES.has(failure.code)
    ? failure.code
    : "stream_interrupted";
  return {
    code,
    isConnection: failure?.is_connection ?? latest.error_type === "connection_lost",
    diagnosticSummary: latest.safe_summary,
  };
}

function timestamp(value: string | undefined): number {
  const parsed = value ? Date.parse(value) : Number.NaN;
  return Number.isFinite(parsed) ? parsed : 0;
}
