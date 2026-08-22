import { describe, expect, it } from "vitest";
import { latestTerminalFailure } from "./agent-session-failure";
import type { AgentSession } from "@/types/agent";

function session(): AgentSession {
  return {
    id: "session-fixture", name: "Fixture", created_at: "2026-08-22T12:00:00Z",
    model: "grok-4.6", provider: "xai-oauth", thinking_enabled: true,
    accumulated_tokens: 0,
    messages: [{
      id: "user-1", role: "user", content: "Bonjour", files: [],
      timestamp: "2026-08-22T12:50:32Z",
    }],
    diagnostic_runs: [{
      request_id: "request-1", generation: 1, status: "failed", severity: "error",
      started_at: "2026-08-22T12:50:32Z", updated_at: "2026-08-22T12:50:48Z",
      ended_at: "2026-08-22T12:50:48Z", phase: "retrying", error_type: "provider_error",
      safe_summary: "Interruption pendant retrying (provider_error).",
    }],
    stream_failures: [{
      code: "provider_error", occurred_at: "2026-08-22T12:50:48Z", is_connection: false,
    }],
  };
}

describe("latestTerminalFailure", () => {
  it("restaure un échec terminal sans fabriquer de message assistant", () => {
    const fixture = session();
    expect(latestTerminalFailure(fixture)).toEqual({
      code: "stream_interrupted",
      isConnection: false,
      diagnosticSummary: "Interruption pendant retrying (provider_error).",
    });
    expect(fixture.messages).toHaveLength(1);
  });

  it("ignore l’échec lorsqu’une réponse assistant plus récente existe", () => {
    const fixture = session();
    fixture.messages.push({
      id: "assistant-1", role: "assistant", content: "Réponse", files: [],
      timestamp: "2026-08-22T12:51:00Z",
    });
    expect(latestTerminalFailure(fixture)).toBeNull();
  });

  it("conserve un code provider stable connu", () => {
    const fixture = session();
    fixture.stream_failures![0].code = "provider_quota_exhausted";
    expect(latestTerminalFailure(fixture)?.code).toBe("provider_quota_exhausted");
  });
});
