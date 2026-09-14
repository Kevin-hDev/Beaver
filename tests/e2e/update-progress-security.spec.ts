import assert from "node:assert/strict";
import { resolve } from "node:path";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

const PROVIDER = "openai";
const updateProgressTest = process.platform === "linux" ? it.skip : it;

describe("update progress IPC boundary", () => {
  updateProgressTest("keeps update state across a secure secondary-window reopen", async () => {
    await waitForTauriBridge();
    const mainHandle = await browser.getWindowHandle();
    await invokeTauri("delete_api_key", { provider: PROVIDER });
    await invokeTauri("e2e_seed_update_operation");

    await switchToUpdateWindow();
    await waitForTauriBridge();
    const initialRect = await browser.getWindowRect();
    await browser.setWindowRect(
      initialRect.x + 24,
      initialRect.y + 24,
      initialRect.width,
      initialRect.height,
    );
    const movedRect = await browser.getWindowRect();
    await browser.pause(300);
    await browser.waitUntil(async () => await browser.$$(".upw-line").length === 4, {
      timeoutMsg: "update progress snapshot did not render",
    });
    assert.match(await browser.$("body").getText(), /Beaver E2E/);
    if (process.env.E2E_ARTIFACT_DIR) {
      await browser.saveScreenshot(resolve(process.env.E2E_ARTIFACT_DIR, "update-progress-window.png"));
    }

    const error = await browser.execute(async (provider) => {
      const invoke = window.__TAURI__?.core?.invoke;
      if (!invoke) return "tauri-bridge-missing";
      try {
        await invoke("set_api_key", {
          provider,
          key: "e2e-value-that-must-not-be-saved",
          connection: null,
        });
        return null;
      } catch (caught) {
        return typeof caught === "string" ? caught : null;
      }
    }, PROVIDER);
    assert.equal(error, "command-not-available");

    await browser.closeWindow();
    await browser.switchToWindow(mainHandle);
    assert.equal(await invokeTauri<boolean>("has_api_key", { provider: PROVIDER }), false);

    await invokeTauri("show_update_progress_window");
    const reopenedHandle = await switchToUpdateWindow();
    await waitForTauriBridge();
    assert.match(await browser.$("body").getText(), /Beaver E2E/);
    const reopenedRect = await browser.getWindowRect();
    const positionEvidence = JSON.stringify({ moved: movedRect, reopened: reopenedRect });
    if (Math.abs(reopenedRect.x - movedRect.x) > 2 || Math.abs(reopenedRect.y - movedRect.y) > 2) {
      throw new Error(positionEvidence);
    }

    await browser.switchToWindow(mainHandle);
    await invokeTauri("e2e_seed_update_operation");
    await browser.switchToWindow(reopenedHandle);
    await browser.waitUntil(async () => browser.$(".upw-finished").isExisting(), {
      timeoutMsg: "terminal update state did not reach the secondary window",
    });
    await browser.closeWindow();
    await browser.switchToWindow(mainHandle);
  });
});

async function switchToUpdateWindow(): Promise<string> {
  let updateHandle: string | undefined;
  await browser.waitUntil(async () => {
    for (const handle of await browser.getWindowHandles()) {
      await browser.switchToWindow(handle);
      if ((await browser.getUrl()).endsWith("/update-window.html")) {
        updateHandle = handle;
        return true;
      }
    }
    return false;
  }, { timeoutMsg: "update progress window did not open" });
  assert.ok(updateHandle);
  return updateHandle;
}
