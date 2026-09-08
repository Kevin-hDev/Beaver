import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import { completeOnboarding } from "./onboarding-flow";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";
import { startFaviconFixture } from "./browser-favicon-fixture";
import type { BrowserSessionState } from "../../src/components/internal-browser/browser-types";

const proof = process.env.E2E_ARTIFACT_DIR!;

describe("Native browser favicons", () => {
  it("renders icons, rejects stale results and omits favicon cookies", async () => {
    const fixture = await startFaviconFixture();
    try {
      await waitForTauriBridge();
      await invokeTauri("e2e_browser_session_key_fixture");
      await completeOnboarding();
      const session = await invokeTauri<{ id: string }>("create_agent_session", {
        name: "Favicon acceptance", model: "local-test", provider: "ollama", projectId: null,
        reasoningMode: null, supportsThinking: null, fastModeEnabled: false,
      });
      await browser.refresh();
      await waitForTauriBridge();
      await $(".conv-item[data-drag-id='" + session.id + "']").waitForDisplayed();
      await $(".conv-item[data-drag-id='" + session.id + "']").click();
      await writeFile(join(proof, "native-ready.json"), JSON.stringify({ driver: browser.sessionId, conversation: session.id }));
      await browser.waitUntil(async () => {
        const capability = await invokeTauri<{ status: string }>("browser_capability");
        await writeFile(join(proof, "native-capability.json"), JSON.stringify(capability));
        return capability.status === "ready";
      }, { timeout: 300_000, interval: 1000, timeoutMsg: "CEF unavailable; check macOS Keychain" });
      await $('.tab-action-btn:has(svg rect[width="6"])').click();
      await $$('.asp-mode-item')[2].click();
      await $('.ib-root').waitForDisplayed();
      const state = await invokeTauri<BrowserSessionState>("browser_open_session", { conversationId: session.id });
      const tabId = state.activeTabId;
      const image = () => $(`[data-browser-tab-id='${tabId}'] img`);
      const iconSrc = async () => await image().isExisting() ? image().getAttribute("src") : null;
      const navigate = async (path: string) => {
        const input = $('.ib-address-input');
        await input.setValue(path.startsWith("http") ? path : fixture.base + path);
        await browser.keys("Enter");
      };
      const waitIcon = async () => {
        await browser.waitUntil(async () => (await iconSrc())?.startsWith("data:image/png;base64,") === true,
          { timeout: 15000, timeoutMsg: "Site favicon did not reach the tab" });
        return iconSrc();
      };
      await navigate('/a');
      const red = await waitIcon();
      await navigate('/b');
      await browser.waitUntil(async () => { const src = await iconSrc(); return src?.startsWith("data:") && src !== red; }, { timeout: 15000 });
      const blue = await iconSrc();
      await navigate('/dynamic');
      await browser.waitUntil(async () => await iconSrc() === red, { timeout: 10000 });
      await browser.waitUntil(async () => await iconSrc() === blue, { timeout: 10000 });
      await navigate('/slow');
      await browser.waitUntil(() => fixture.requests.some((r) => r.path === '/icon/slow'), { timeout: 10000 });
      await navigate('/b');
      await browser.waitUntil(async () => await iconSrc() === blue, { timeout: 10000 });
      await navigate('/missing');
      await browser.waitUntil(() => fixture.requests.some((r) => r.path === '/icon/missing'), { timeout: 10000 });
      assert.ok(!(await iconSrc())?.startsWith('data:'));
      await navigate('/timeout');
      await browser.waitUntil(() => fixture.requests.some((r) => r.path === '/icon/timeout'), { timeout: 10000 });
      await browser.pause(8000);
      assert.ok(!(await iconSrc())?.startsWith('data:'));
      await navigate('/a'); await waitIcon();
      await browser.waitUntil(() => fixture.requests.some((r) => r.path === '/cookie-proof' && r.cookie.includes('fixture_page=1')), { timeout: 10000 });
      assert.ok(fixture.requests.filter((r) => r.path.startsWith('/icon/')).every((r) => r.cookie === ''));
      assert.ok(fixture.requests.filter((r) => r.path === '/cookie-proof').every((r) => !r.cookie.includes('favicon_response')));
      assert.ok(fixture.peak() <= 4);
      await browser.saveScreenshot(join(proof, 'native-favicon-dark.png'));
      await browser.execute(() => { localStorage.setItem('clgo-theme', 'light'); localStorage.setItem('clgo-theme-base', 'light'); });
      await browser.refresh(); await waitForTauriBridge();
      await $(".conv-item[data-drag-id='" + session.id + "']").click();
      await waitIcon();
      await browser.saveScreenshot(join(proof, 'native-favicon-light.png'));
      for (const site of ['https://www.google.com/', 'https://openai.com/']) {
        await navigate(site); await waitIcon();
        await browser.saveScreenshot(join(proof, site.includes('google') ? 'native-google.png' : 'native-openai.png'));
      }
      await writeFile(join(proof, 'native-fixture-results.json'), JSON.stringify({ requests: fixture.requests, peak: fixture.peak() }, null, 2));
    } finally { await fixture.close(); }
  });
});
