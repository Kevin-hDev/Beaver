import { afterEach, describe, expect, it, vi } from "vitest";
import {
  flushFrameNotify,
  scheduleFrameNotify,
  shouldDeferStreamEvent,
} from "@/hooks/agent-stream-notify";
import { createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { StreamRecord } from "@/hooks/agent-stream-cleanup";

function makeRecord(): StreamRecord {
  return {
    state: createManagedStreamState([], 0),
    history: [],
    subscribers: new Map(),
    nextSubscriberId: 1,
    cleanupTimer: null,
    notifyHandle: null,
    started: true,
    backendOwnsPersistence: false,
    isSubagentBackendStream: false,
    activeGeneration: null,
    cancelledGenerations: [],
    cancelledWithoutGeneration: false,
    persistenceQueue: Promise.resolve(),
    persistencePending: false,
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("shouldDeferStreamEvent", () => {
  it("diffère seulement les fragments texte et thinking", () => {
    expect(shouldDeferStreamEvent({ event: "token", data: { content: "a", tokenCount: 1, tps: 0 } })).toBe(true);
    expect(shouldDeferStreamEvent({ event: "thinking", data: { content: "a" } })).toBe(true);
    expect(shouldDeferStreamEvent({ event: "done", data: { evalCount: 0, evalDurationNs: 0, finalTps: 0, promptTokens: 0, contextTokens: 0 } })).toBe(false);
  });
});

describe("scheduleFrameNotify", () => {
  it("regroupe plusieurs notifications dans une seule frame", () => {
    const callbacks: FrameRequestCallback[] = [];
    vi.stubGlobal("requestAnimationFrame", vi.fn((cb: FrameRequestCallback) => {
      callbacks.push(cb);
      return callbacks.length;
    }));
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
    const record = makeRecord();
    const dispatch = vi.fn();

    scheduleFrameNotify(record, dispatch);
    scheduleFrameNotify(record, dispatch);

    expect(dispatch).not.toHaveBeenCalled();
    callbacks[0](performance.now());
    expect(dispatch).toHaveBeenCalledTimes(1);
  });

  it("flush annule la frame en attente et notifie tout de suite", () => {
    const callbacks: FrameRequestCallback[] = [];
    const cancel = vi.fn();
    vi.stubGlobal("requestAnimationFrame", vi.fn((cb: FrameRequestCallback) => {
      callbacks.push(cb);
      return 7;
    }));
    vi.stubGlobal("cancelAnimationFrame", cancel);
    const record = makeRecord();
    const dispatch = vi.fn();

    scheduleFrameNotify(record, dispatch);
    flushFrameNotify(record, dispatch);

    expect(cancel).toHaveBeenCalledWith(7);
    expect(dispatch).toHaveBeenCalledTimes(1);
    expect(record.notifyHandle).toBeNull();
  });
});
