import { beforeEach, describe, expect, it } from "vitest";
import { applyStreamEvent } from "../agent-chat-stream-callbacks";
import { records, snapshot, startStreamRecord } from "../agent-stream-records";

describe("agent stream permission records", () => {
  beforeEach(() => records.clear());

  it("preserves the complete extension permission display in snapshots", () => {
    const record = startStreamRecord("session", [], 0, "chat");
    const request = {
      id: "request", toolName: "plugin.tool", arguments: {},
      extensionId: "plugin-id", extensionName: "Plugin",
      effectClass: "secret" as const, actionSummary: "{\"value\":\"[REDACTED]\"}",
      allowSession: false,
    };
    record.state = applyStreamEvent(record.state, {
      event: "permissionRequest",
      data: request,
    }).state;

    expect(snapshot(record.state).pendingPermissions).toEqual([request]);
  });
});
