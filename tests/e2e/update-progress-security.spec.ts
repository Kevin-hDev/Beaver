import assert from "node:assert/strict";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

const PROVIDER = "openai";

describe("update progress IPC boundary", () => {
  it("rejects a sensitive command from the real secondary webview", async () => {
    await waitForTauriBridge();
    const mainHandle = await browser.getWindowHandle();
    await invokeTauri("delete_api_key", { provider: PROVIDER });
    await invokeTauri("e2e_seed_update_operation");

    await browser.waitUntil(async () => (await browser.getWindowHandles()).length === 2, {
      timeoutMsg: "update progress window did not open",
    });
    const updateHandle = (await browser.getWindowHandles()).find((handle) => handle !== mainHandle);
    assert.ok(updateHandle);
    await browser.switchToWindow(updateHandle);
    await waitForTauriBridge();

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
  });
});
