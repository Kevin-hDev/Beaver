import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { runAdvancedCleanups } from "./advanced-cleanup";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";

describe("advanced cleanup deadline", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("isolates throws and rejections and releases its timer when callbacks finish", async () => {
    const last = vi.fn();
    await runAdvancedCleanups([
      () => { throw new Error("fixture"); },
      () => Promise.reject(new Error("fixture")), last,
    ], performance.now() + UI_LIMITS.maxAdvancedCleanupMs);
    expect(last).toHaveBeenCalledOnce();
    expect(vi.getTimerCount()).toBe(0);
  });

  it("uses the remaining absolute budget and handles a rejection after expiry", async () => {
    let reject!: (error: Error) => void;
    const deadline = performance.now() + UI_LIMITS.maxAdvancedCleanupMs;
    await vi.advanceTimersByTimeAsync(UI_LIMITS.maxAdvancedCleanupMs - 1);
    const done = vi.fn();
    const finishing = runAdvancedCleanups([
      () => new Promise<void>((_resolve, failure) => { reject = failure; }),
      () => new Promise<void>(() => {}),
    ], deadline).then(done);
    await vi.advanceTimersByTimeAsync(1);
    await finishing;
    expect(done).toHaveBeenCalledOnce();
    expect(vi.getTimerCount()).toBe(0);
    reject(new Error("late fixture"));
    await Promise.resolve();
  });
});
