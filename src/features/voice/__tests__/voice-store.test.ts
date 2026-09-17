import { expect, it, vi } from "vitest";
import {
  acceptVoiceSnapshot,
  resetVoiceStoreForTests,
  subscribeVoiceSnapshots,
} from "../voice-store";

it("keeps existing subscribers when the bounded listener set is full", () => {
  resetVoiceStoreForTests();
  const first = vi.fn();
  const cleanups = [subscribeVoiceSnapshots(first)];
  for (let index = 1; index < 64; index += 1) {
    cleanups.push(subscribeVoiceSnapshots(vi.fn()));
  }
  expect(() => subscribeVoiceSnapshots(vi.fn()))
    .toThrow("active_view_subscription_limit_reached");
  acceptVoiceSnapshot({
    revision: 1,
    phase: "idle",
    operation: null,
    recovery: null,
    delivery: null,
    trialResult: null,
    error: null,
  });
  expect(first).toHaveBeenCalledOnce();
  cleanups.pop()?.();
  const replacement = subscribeVoiceSnapshots(vi.fn());
  replacement();
  cleanups.forEach((cleanup) => cleanup());
});
