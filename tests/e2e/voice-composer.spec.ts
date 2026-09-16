import assert from "node:assert/strict";
import { completeOnboarding } from "./onboarding-flow";

describe("voice composer platform boundary", () => {
  (process.platform === "linux" ? it : it.skip)("keeps the microphone absent on Linux", async () => {
    await completeOnboarding();
    assert.equal(await $$(".vc-mic").length, 0);
    assert.equal(await $$(".vc-elsewhere").length, 0);
  });
});
