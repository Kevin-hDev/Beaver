import { randomUUID } from "node:crypto";
import { createServer, type Server } from "node:http";
import {
  runtimeRootForBinary,
  waitForOwnedCefHelper,
  waitForOwnedProcessesToExit,
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

const required = process.env.E2E_REQUIRE_CEF_SMOKE === "1";
if (required && !["win32", "darwin"].includes(process.platform)) {
  throw new Error("Native CEF smoke is unsupported on this platform");
}
const nativeTest = required ? it : it.skip;

describe("native CEF shutdown", () => {
  nativeTest("loads a page in a real helper and leaves no owned process", async () => {
    const binaryPath = process.env.E2E_APP_BINARY;
    if (!binaryPath) throw new Error("E2E app binary is not configured");
    const runtimeRoot = runtimeRootForBinary(process.platform, binaryPath);
    const server = await startPageServer();
    try {
      await completeOnboarding();
      const url = serverUrl(server);
      await browser.waitUntil(async () => (
        (await invokeTauri<BrowserCapability>("browser_capability")).status === "ready"
      ), { timeout: 15_000, interval: 100 });

      const conversationId = randomUUID();
      const session = await invokeTauri<BrowserSession>("browser_open_session", {
        conversationId,
      });
      await invokeTauri("browser_surface", {
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
      });

      await waitForOwnedCefHelper({ root: runtimeRoot });
      await browser.waitUntil(async () => {
        const current = await invokeTauri<BrowserSession>("browser_open_session", {
          conversationId,
        });
        const tab = current.tabs.find(({ id }) => id === current.activeTabId);
        return tab?.title === "Beaver CEF smoke" && !tab.loading;
      }, { timeout: 15_000, interval: 100 });

      await invokeTauri("e2e_request_exit");
      await browser.deleteSession();
      (browser as unknown as { sessionId?: string }).sessionId = undefined;
      await waitForOwnedProcessesToExit({ root: runtimeRoot });
    } finally {
      await closeServer(server);
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
  });
}
