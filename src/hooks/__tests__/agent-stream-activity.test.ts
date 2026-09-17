import { describe, expect, it, vi } from "vitest";
import { createManagedStreamState } from "../agent-chat-stream-types";
import { emitStreamActivity, subscribeStreamActivity } from "../agent-stream-activity";

describe("agent stream activity subscriptions", () => {
  it("refuse explicitement la saturation et accepte après libération", () => {
    const first = vi.fn();
    const cleanups = [subscribeStreamActivity(first)];
    for (let index = 1; index < 16; index += 1) {
      cleanups.push(subscribeStreamActivity(vi.fn()));
    }

    expect(() => subscribeStreamActivity(vi.fn()))
      .toThrow("Active view subscription limit reached");
    emitStreamActivity("session", createManagedStreamState([], 0));
    expect(first).toHaveBeenCalledOnce();

    cleanups.pop()?.();
    const replacement = subscribeStreamActivity(vi.fn());
    replacement();
    cleanups.forEach((cleanup) => cleanup());
  });
});
