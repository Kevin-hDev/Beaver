import { describe, expect, it, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "../agent-chat-stream-callbacks";
import { restoredFailureState } from "../agent-chat-restored-failure";
import type { AgentSession } from "@/types/agent";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

const extensionErrorCodes = [
  "extensions_registry_version_unsupported",
  "extensions_registry_unavailable",
  "extensions_registry_migration_failed",
  "extensions_state_unavailable",
];

function failedSession(code: string): AgentSession {
  return {
    id: "session-fixture", name: "Fixture", created_at: "2026-09-06T12:00:00Z",
    model: "fixture", provider: "fixture", thinking_enabled: false, fast_mode_enabled: false,
    plan_mode_enabled: false, plan_workflow_status: "needs_context",
    is_heartbeat: false, is_gateway: false, working_dir: "", working_dir_managed: false,
    automatic_compression_suspended: false, accumulated_tokens: 0, messages: [],
    diagnostic_runs: [{
      request_id: "request-1", generation: 1, status: "failed", severity: "error",
      started_at: "2026-09-06T12:00:00Z", updated_at: "2026-09-06T12:00:01Z",
      ended_at: "2026-09-06T12:00:01Z", phase: "request_start", error_type: "unknown",
      safe_summary: "Interruption pendant request_start (unknown).",
    }],
    stream_failures: [{ code, occurred_at: "2026-09-06T12:00:01Z", is_connection: false }],
  };
}

describe("erreurs d'extensions", () => {
  it.each(extensionErrorCodes)("traduit l'erreur directe %s sans repli générique", (code) => {
    const result = applyStreamEvent(createManagedStreamState([], 0), {
      event: "error", data: {
        message: code,
        diagnostic: {
          requestId: "request-1", phase: "request_start", errorType: "unknown",
          safeSummary: "Interruption pendant request_start (unknown).",
        },
      },
    });
    expect(result.state.error).toBe(`extensions.errors.codes.${code}`);
    expect(result.state.isConnectionError).toBe(false);
    expect(result.state.diagnosticSummary).toBeUndefined();
    expect(result.state.isStreaming).toBe(false);
    expect(result.assistantMessage).toBeUndefined();
  });

  it.each(extensionErrorCodes)("restaure l'erreur %s sans exposer le diagnostic technique", (code) => {
    expect(restoredFailureState(failedSession(code))).toEqual({
      error: `extensions.errors.codes.${code}`,
      isConnectionError: false,
      diagnosticSummary: undefined,
    });
  });
});

it("conserve la réponse partielle et traduit une erreur stable reçue en cours de tour", () => {
  const streaming = applyStreamEvent(createManagedStreamState([], 0), {
    event: "token", data: { content: "Début de réponse", tokenCount: 3, tps: 1 },
  }).state;
  const result = applyStreamEvent(streaming, {
    event: "error", data: { message: "extensions_state_unavailable" },
  });
  expect(result.state.error).toBe("extensions.errors.codes.extensions_state_unavailable");
  expect(result.state.diagnosticSummary).toBeUndefined();
  expect(result.assistantMessage?.content).toContain("Début de réponse");
});
