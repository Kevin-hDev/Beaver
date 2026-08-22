import i18n from "@/i18n";
import { KNOWN_ERROR_KEYS } from "@/lib/agent-error-codes";
import { latestTerminalFailure } from "@/lib/agent-session-failure";
import type { AgentSession } from "@/types/agent";
import type { ChatState } from "./agent-chat-stream-types";

export function restoredFailureState(session: AgentSession): Pick<
  ChatState,
  "error" | "isConnectionError" | "diagnosticSummary"
> {
  const failure = latestTerminalFailure(session);
  if (!failure) {
    return { error: undefined, isConnectionError: false, diagnosticSummary: undefined };
  }
  const key = KNOWN_ERROR_KEYS[failure.code] ?? "errors.streamInterrupted";
  return {
    error: i18n.t(key),
    isConnectionError: failure.isConnection,
    // Les détails techniques persistent pour le diagnostic, mais restent hors de l'UI.
    diagnosticSummary: undefined,
  };
}
