import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { failSession } from "../agent-stream-failure";
import { getRecord, records, startStreamRecord } from "../agent-stream-records";

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
});
