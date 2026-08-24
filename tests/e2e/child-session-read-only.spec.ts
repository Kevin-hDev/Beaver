import assert from "node:assert/strict";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";
import { SUBAGENT_READ_ONLY_CODE } from "../../src/lib/admission-error";

interface ChildReadOnlyOutcome {
  code: string;
  sessionUnchanged: boolean;
  requestStartUnchanged: boolean;
  activeStreamAbsent: boolean;
}

describe("child session read-only boundary", () => {
  it("rejects chat_stream before any persisted or runtime mutation", async () => {
    // Readiness may lag behind WebDriver startup; only poll the bridge so the
    // mutating IPC command itself is still executed exactly once.
    await waitForTauriBridge();
    const outcome = await invokeTauri<ChildReadOnlyOutcome>(
      "e2e_verify_child_chat_stream_read_only",
    );

    assert.deepEqual(outcome, {
      code: SUBAGENT_READ_ONLY_CODE,
      sessionUnchanged: true,
      requestStartUnchanged: true,
      activeStreamAbsent: true,
    });
  });
});
