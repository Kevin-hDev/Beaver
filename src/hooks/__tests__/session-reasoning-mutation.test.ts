import { describe, expect, it, vi } from "vitest";
import {
  awaitPendingReasoning,
  runReasoningMutation,
} from "../session-reasoning-mutation";

describe("session reasoning mutation", () => {
  it("sérialise une mutation puis expose sa fin au prochain envoi", async () => {
    let release = () => {};
    let markEntered = () => {};
    const entered = new Promise<void>((resolve) => { markEntered = resolve; });
    const first = runReasoningMutation("serialized", () => {
      markEntered();
      return new Promise<void>((resolve) => { release = resolve; });
    });
    const secondMutation = vi.fn();
    const second = runReasoningMutation("serialized", () => {
      secondMutation();
      return Promise.resolve();
    });
    await entered;
    expect(secondMutation).not.toHaveBeenCalled();

    release();
    await awaitPendingReasoning("serialized");
    await Promise.all([first, second]);
    expect(secondMutation).toHaveBeenCalledOnce();
  });

  it("borne le nombre de sessions avec une mutation en attente", async () => {
    const releases: Array<() => void> = [];
    const tasks = Array.from({ length: 64 }, (_, index) =>
      runReasoningMutation(`bounded-${index}`, () => new Promise<void>((resolve) => {
        releases.push(resolve);
      })),
    );
    await Promise.resolve();

    await expect(runReasoningMutation("overflow", () => Promise.resolve()))
      .rejects.toThrow("session-update-unavailable");
    await vi.waitFor(() => expect(releases).toHaveLength(64));
    releases.forEach((release) => release());
    await Promise.all(tasks);
  });
});
