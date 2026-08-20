import assert from "node:assert/strict";
import { invokeTauri } from "./tauri-invoke";
import { SUBAGENT_READ_ONLY_CODE } from "../../src/lib/admission-error";

interface ChildReadOnlyOutcome {
  code: string;
  sessionUnchanged: boolean;
  requestStartUnchanged: boolean;
  activeStreamAbsent: boolean;
}

describe("child session read-only boundary", () => {
  it("rejects chat_stream before any persisted or runtime mutation", async () => {
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
