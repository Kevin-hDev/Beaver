import assert from "node:assert/strict";
import { APP_SHORTCUTS } from "../../src/lib/app-shortcuts";
import { completeOnboarding } from "./onboarding-flow";

describe("first launch", () => {
  it("completes onboarding in the isolated desktop app", async () => {
    await completeOnboarding();
  });

  it("opens the complete shortcut reference from the application shortcut", async () => {
    const modifier = process.platform === "darwin" ? "Meta" : "Control";
    await browser.keys([modifier, ",", "NULL"]);

    const settingsTabs = $$(".settings-subtab");
    await settingsTabs[0].waitForDisplayed();
    assert.ok(await settingsTabs.length >= 3);
    await settingsTabs[2].click();

    await $(".scs-row").waitForDisplayed();
    const rows = $$(".scs-row");
    assert.equal(await rows.length, APP_SHORTCUTS.length);
  });
});
