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
      code: "provider_error",
      isConnection: false,
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

  it.each([
    "connection_lost",
    "timeout",
    "provider_overloaded",
    "provider_error",
    "max_turns",
    "circuit_breaker",
    "tool_error",
    "stream_error",
  ])("conserve le code Rust persiste %s", (code) => {
    const fixture = session();
    fixture.stream_failures![0].code = code;
    expect(latestTerminalFailure(fixture)?.code).toBe(code);
  });

  it("remplace un code inconnu par l'erreur generique persistable", () => {
    const fixture = session();
    fixture.stream_failures![0].code = "provider_internal_secret";

    expect(latestTerminalFailure(fixture)?.code).toBe("stream_interrupted");
  });

  it.each(["completed", "cancelled"])("ignore une execution %s", (status) => {
    const fixture = session();
    fixture.diagnostic_runs![0].status = status;
    expect(latestTerminalFailure(fixture)).toBeNull();
  });

  it("n'affiche pas un ancien echec lorsqu'une execution est encore active", () => {
    const fixture = session();
    fixture.diagnostic_runs!.push({
      request_id: "request-2", generation: 2, status: "running", severity: "info",
      started_at: "2026-08-22T12:51:00Z", updated_at: "2026-08-22T12:51:01Z",
      phase: "streaming",
    });
    expect(latestTerminalFailure(fixture)).toBeNull();
  });
});
