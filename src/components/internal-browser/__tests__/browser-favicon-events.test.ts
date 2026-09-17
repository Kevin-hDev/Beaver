import { describe, expect, it } from "vitest";
import { parseFaviconSnapshot } from "../browser-favicon-events";

const id = "1".repeat(32);
const icon = { tabId: id, pngBase64: "iVBORw0KGgo=" };
const snapshot = { eventVersion: 1, revision: 2, conversationId: "a", icons: [icon] };

describe("favicon snapshot boundary", () => {
  it("route uniquement la version et la conversation attendues", () => {
    expect(parseFaviconSnapshot(snapshot, "a")).toEqual(snapshot);
    for (const patch of [{ eventVersion: 2 }, { conversationId: "b" }]) {
      expect(parseFaviconSnapshot({ ...snapshot, ...patch }, "a")).toBeNull();
    }
  });
  it("rejette les enveloppes qui ne permettent pas le routage", () => {
    for (const value of [null, "text", [], 42, {}]) {
      expect(parseFaviconSnapshot(value, "a")).toBeNull();
    }
  });
  it("accepts an empty replacement snapshot to remove stale icons", () => {
    expect(parseFaviconSnapshot({ ...snapshot, icons: [] }, "a")?.icons).toEqual([]);
  });
});
