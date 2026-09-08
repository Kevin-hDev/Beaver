import { describe, expect, it } from "vitest";
import { parseFaviconSnapshot } from "../browser-favicon-events";
import { MAX_FAVICON_PNG_BYTES } from "../browser-favicon-contract";

const id = "1".repeat(32);
const icon = { tabId: id, pngBase64: "iVBORw0KGgo=" };
const snapshot = { eventVersion: 1, revision: 2, conversationId: "a", icons: [icon] };

describe("favicon snapshot boundary", () => {
  it("accepts only the matching version and conversation with a safe revision", () => {
    expect(parseFaviconSnapshot(snapshot, "a")).toEqual(snapshot);
    for (const patch of [{ eventVersion: 2 }, { revision: -1 }, { revision: Infinity },
      { revision: Number.MAX_SAFE_INTEGER + 1 }, { revision: 1.5 }, { conversationId: "b" }]) {
      expect(parseFaviconSnapshot({ ...snapshot, ...patch }, "a")).toBeNull();
    }
  });
  it("rejects oversized collections, duplicates, bad IDs and non-PNG data", () => {
    for (const icons of [Array(11).fill(icon), [icon, icon], [{ ...icon, tabId: "../bad" }],
      [{ ...icon, pngBase64: "data:image/png;base64,iVBORw0KGgo=" }],
      [{ ...icon, pngBase64: btoa("<svg></svg>") }], [{ ...icon, pngBase64: "iVBORw0KGgo?" }],
      [{ ...icon, pngBase64: btoa("\x89PNG\r\n\x1a\n" + "x".repeat(MAX_FAVICON_PNG_BYTES)) }]]) {
      expect(parseFaviconSnapshot({ ...snapshot, icons }, "a")).toBeNull();
    }
  });
  it("accepts an empty replacement snapshot to remove stale icons", () => {
    expect(parseFaviconSnapshot({ ...snapshot, icons: [] }, "a")?.icons).toEqual([]);
  });
});
