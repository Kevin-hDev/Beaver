import { describe, expect, it } from "vitest";
import {
  cleanupSessionActivity,
  reduceSessionActivity,
  type SessionActivityState,
} from "../use-session-activity-indicators";
import type { StreamActivity } from "../agent-stream-activity";

const visibleIds = new Set(["s1", "s2"]);

function state(running: string[] = [], unread: string[] = []): SessionActivityState {
  return { runningIds: new Set(running), unreadIds: new Set(unread) };
}

function activity(sessionId: string, isStreaming: boolean, completed = false): StreamActivity {
  return { sessionId, isStreaming, completed, updatedAt: 1 };
}

describe("reduceSessionActivity", () => {
  it("marque une session comme en cours et retire son point terminé", () => {
    const result = reduceSessionActivity(
      state([], ["s1"]),
      activity("s1", true),
      null,
      visibleIds,
    );

    expect(result.runningIds.has("s1")).toBe(true);
    expect(result.unreadIds.has("s1")).toBe(false);
  });

  it("marque une session terminée ailleurs comme non consultée", () => {
    const result = reduceSessionActivity(
      state(["s1"]),
      activity("s1", false, true),
      "s2",
      visibleIds,
    );

    expect(result.runningIds.has("s1")).toBe(false);
    expect(result.unreadIds.has("s1")).toBe(true);
  });

  it("ne marque pas la session terminée si elle est déjà active", () => {
    const result = reduceSessionActivity(
      state(["s1"]),
      activity("s1", false, true),
      "s1",
      visibleIds,
    );

    expect(result.runningIds.has("s1")).toBe(false);
    expect(result.unreadIds.has("s1")).toBe(false);
  });

  it("nettoie les sessions en cours invisibles sans masquer les points terminés", () => {
    const result = cleanupSessionActivity(
      state(["s1", "hidden"], ["s2", "hidden"]),
      visibleIds,
      "s2",
    );

    expect([...result.runningIds]).toEqual(["s1"]);
    expect([...result.unreadIds]).toEqual(["s2", "hidden"]);
  });

  it("conserve les points terminés quand la liste remonte sans session chargée", () => {
    const result = cleanupSessionActivity(
      state([], ["s1"]),
      new Set(),
      null,
    );

    expect([...result.unreadIds]).toEqual(["s1"]);
  });
});
