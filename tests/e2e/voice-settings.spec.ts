import assert from "node:assert/strict";
import { completeOnboarding } from "./onboarding-flow";

describe("voice settings platform boundary", () => {
  (process.platform === "linux" ? it : it.skip)("keeps voice settings absent on Linux", async () => {
    await completeOnboarding();
    assert.equal(await $$(".vset-models").length, 0);
  });
});
