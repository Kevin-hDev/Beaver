import { randomUUID } from "node:crypto";
import { createServer, type Server } from "node:http";
import { createNativeJourney } from "../../scripts/e2e/native-journey-deadline.mjs";
import {
  runtimeRootForBinary,
  waitForOwnedCefHelper,
  waitForOwnedProcessesToExit,
  waitForProcessIdsToExit,
} from "../../scripts/e2e/native-cef-observer.mjs";
import { completeOnboarding } from "./onboarding-flow";
import { invokeTauri } from "./tauri-invoke";

interface BrowserCapability {
  status: "ready" | "unavailable" | "hidden";
}

interface BrowserTab {
  id: string;
  title: string;
  loading: boolean;
}

interface BrowserSession {
  activeTabId: string;
  tabs: BrowserTab[];
}

interface NativeWebViews {
  dedicatedPids: number[];
  sharedSystemCount: number;
}

const required = process.env.E2E_REQUIRE_CEF_SMOKE === "1";
if (required && !["win32", "darwin"].includes(process.platform)) {
  throw new Error("Native CEF smoke is unsupported on this platform");
}
const nativeTest = required ? it : it.skip;

describe("native CEF shutdown", () => {
  nativeTest("loads a page in a real helper and leaves no owned process", async () => {
    const journey = createNativeJourney();
    const binaryPath = process.env.E2E_APP_BINARY;
    if (!binaryPath) throw new Error("E2E app binary is not configured");
    const runtimeRoot = runtimeRootForBinary(process.platform, binaryPath);
    let server: Server | undefined;
    try {
      server = await journey.run("page_server_start", () => startPageServer());
      await journey.run("onboarding", () => completeOnboarding());
      const nativeWebViews = await journey.run("native_webviews", async () => {
        const observation = await invokeTauri<NativeWebViews>("e2e_native_webviews");
        if (process.platform === "win32" && observation.dedicatedPids.length === 0) {
          throw new Error("Native WebView observation failed");
        }
        if (process.platform === "darwin" && observation.sharedSystemCount === 0) {
          throw new Error("Native WebView observation failed");
        }
        return observation;
      });
      const url = serverUrl(server);
      await journey.run("browser_capability", ({ timeoutMs }) => (
        browser.waitUntil(async () => (
          (await invokeTauri<BrowserCapability>("browser_capability")).status === "ready"
        ), { timeout: timeoutMs, interval: 100 })
      ));

      const conversationId = randomUUID();
      const session = await journey.run("browser_session_open", () => (
        invokeTauri<BrowserSession>("browser_open_session", { conversationId })
      ));
      await journey.run("browser_surface", () => (
        invokeTauri("browser_surface", {
          request: {
            conversationId,
            tabId: session.activeTabId,
            url,
            bounds: {
              x: 20,
              y: 80,
              width: 480,
              height: 320,
              visible: true,
              generation: 1,
            },
          },
        })
      ));

      await journey.run("cef_helper_start", ({ timeoutMs }) => (
        waitForOwnedCefHelper({ root: runtimeRoot, timeoutMs })
      ));
      await journey.run("page_load", ({ timeoutMs }) => (
        browser.waitUntil(async () => {
          const current = await invokeTauri<BrowserSession>("browser_open_session", {
            conversationId,
          });
          const tab = current.tabs.find(({ id }) => id === current.activeTabId);
          return tab?.title === "Beaver CEF smoke" && !tab.loading;
        }, { timeout: timeoutMs, interval: 100 })
      ));

      await journey.run("exit_request", () => invokeTauri("e2e_request_exit"));
      await journey.run("webdriver_release", async () => {
        try {
          await browser.deleteSession();
        } finally {
          (browser as unknown as { sessionId?: string }).sessionId = undefined;
        }
      });
      await journey.run("owned_process_exit", ({ timeoutMs }) => (
        waitForOwnedProcessesToExit({ root: runtimeRoot, timeoutMs })
      ));
      await journey.run("native_webview_exit", ({ timeoutMs }) => (
        waitForProcessIdsToExit({ pids: nativeWebViews.dedicatedPids, timeoutMs })
      ));
    } finally {
      const activeServer = server;
      if (activeServer) {
        await journey.cleanup("page_server_close", () => closeServer(activeServer));
      }
    }
  });
});

function startPageServer(): Promise<Server> {
  const server = createServer((_request, response) => {
    response.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
    response.end("<!doctype html><title>Beaver CEF smoke</title><p>ready</p>");
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => resolve(server));
  });
}

function serverUrl(server: Server): string {
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("E2E server failed");
  return `http://127.0.0.1:${address.port}/`;
}

function closeServer(server: Server): Promise<void> {
  return new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
    server.closeAllConnections();
  });
}
