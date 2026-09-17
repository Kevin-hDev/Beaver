import { describe, expect, it, vi } from "vitest";
import { error as logError } from "@tauri-apps/plugin-log";
import { addBoundedSubscriber } from "./bounded-subscriber";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn().mockResolvedValue(undefined) }));

describe("addBoundedSubscriber", () => {
  it("journalise le registre et sa limite avant de refuser", () => {
    const subscribers = new Map([[1, vi.fn()]]);
    expect(() => addBoundedSubscriber(subscribers, 2, vi.fn(), 1, "fixture"))
      .toThrow("active_view_subscription_limit_reached");
    expect(logError).toHaveBeenCalledWith("active_view_subscription_limit_reached", {
      keyValues: { registry: "fixture", size: "1", limit: "1" },
    });
  });
});
