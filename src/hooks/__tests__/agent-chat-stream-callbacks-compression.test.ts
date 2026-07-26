import { describe, expect, it } from "vitest";
import {
  applyStreamEvent,
  createManagedStreamState,
} from "../agent-chat-stream-callbacks";

describe("compression stream state", () => {
  it("suit le démarrage et la fin dans l'état central du flux", () => {
    const initial = createManagedStreamState([], 0);
    const started = applyStreamEvent(initial, {
      event: "compressing",
      data: { status: "start" },
    }).state;

    expect(started.isCompressing).toBe(true);

    const completed = applyStreamEvent(started, {
      event: "compressing",
      data: { status: "done" },
    }).state;

    expect(completed.isCompressing).toBe(false);
  });
});
