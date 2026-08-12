import { waitForProcessIdsToExit } from "../../scripts/e2e/native-cef-observer.mjs";
import { completeOnboarding } from "./onboarding-flow";
import { invokeTauri } from "./tauri-invoke";

interface NativeWebViews {
  dedicatedPids: number[];
  sharedSystemCount: number;
}

const required = process.env.E2E_REQUIRE_WEBVIEW_SMOKE === "1";
if (required && process.platform !== "linux") {
  throw new Error("Native WebView smoke is reserved for Linux");
}
const nativeTest = required ? it : it.skip;

describe("native Tauri WebView shutdown", () => {
  nativeTest("classifies the real WebView and leaves no dedicated descendant", async () => {
    await completeOnboarding();
    const observation = await invokeTauri<NativeWebViews>("e2e_native_webviews");
    if (observation.dedicatedPids.length === 0) {
      throw new Error("Native WebView observation failed");
    }

    await invokeTauri("e2e_request_exit");
    await browser.deleteSession();
    (browser as unknown as { sessionId?: string }).sessionId = undefined;
    await waitForProcessIdsToExit({ pids: observation.dedicatedPids });
  });
});
