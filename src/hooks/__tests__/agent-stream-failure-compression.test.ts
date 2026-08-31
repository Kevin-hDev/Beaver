import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { failSession } from "../agent-stream-failure";
import { getRecord, records, startStreamRecord } from "../agent-stream-records";
import type { AgentMessage } from "@/types/agent";

function message(role: AgentMessage["role"]): AgentMessage {
  return {
    id: crypto.randomUUID(),
    turn_id: crypto.randomUUID(),
    role,
    content: "contexte persistant",
    timestamp: "2026-08-31T12:00:00Z",
    files: [],
    tokens: 0,
  };
}

describe("échec au démarrage d'une compression", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    records.clear();
    vi.stubGlobal("requestAnimationFrame", vi.fn(() => 1));
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("retire aussi l'état de compression", () => {
    startStreamRecord("session-1", [], 0, "compression");

    failSession("session-1");

    expect(getRecord("session-1")?.state.isStreaming).toBe(false);
    expect(getRecord("session-1")?.state.isCompressing).toBe(false);
  });

  it("conserve l'anneau après l'échec d'une première compression depuis le rechargement", () => {
    startStreamRecord("session-1", [message("user"), message("assistant")], 712, "compression");

    failSession("session-1");

    expect(getRecord("session-1")?.state.contextUsageVisible).toBe(true);
    expect(getRecord("session-1")?.state.sessionTokenCount).toBe(712);
  });
});
