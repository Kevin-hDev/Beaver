import assert from "node:assert/strict";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

describe("voice native platform boundary", () => {
  (process.platform === "linux" ? it : it.skip)("registers no voice command on Linux", async () => {
    await waitForTauriBridge();
    await assert.rejects(() => invokeTauri("voice_get_snapshot"));
  });
});
