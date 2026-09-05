import assert from "node:assert/strict";
import { resolve } from "node:path";
import { performance } from "node:perf_hooks";
import {
  diagnosticProfile, DIAGNOSTIC_POLL_MS, MAX_DIAGNOSTIC_SAMPLES,
  sampleMarker, safeOutcome, verifiedScriptTimeout,
} from "../../scripts/e2e/activation-diagnostic.mjs";
import { EXTENSION_DIAGNOSTIC_TIMEOUT_MS, EXTENSION_UI_SETUP_TIMEOUT_MS } from "../../scripts/e2e/extension-setup-deadline";
import { completeOnboarding } from "./onboarding-flow";
import { initializeExtensionHost } from "./extension-host-setup";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

const ID = "acceptance.standard.complete";
const FIXTURE = resolve("scripts/extensions/fixtures/ui/standard-complete");

describe("isolated Windows activation diagnosis", () => {
  it("measures one activation without replaying its mutation", async function () {
    this.timeout(EXTENSION_DIAGNOSTIC_TIMEOUT_MS);
    const profile = await diagnosticProfile(process.env.CL_GO_CEF_TEST_DATA_DIR);
    const timeout = await verifiedScriptTimeout(browser, EXTENSION_UI_SETUP_TIMEOUT_MS);
    console.log("[activation-diagnostic]", JSON.stringify({phase: "timeouts", ...timeout}));
    const start = performance.now();
    let sampling = true;
    let samples = 0;
    const sampler = (async () => {
      let previous = "";
      while (sampling && samples < MAX_DIAGNOSTIC_SAMPLES) {
        const marker = await sampleMarker(profile);
        const key = JSON.stringify(marker);
        if (key !== previous) {
          console.log("[activation-diagnostic]", JSON.stringify({phase: "marker", ms: Math.round(performance.now() - start), ...marker}));
          previous = key;
        }
        samples += 1;
        await new Promise((done) => setTimeout(done, DIAGNOSTIC_POLL_MS));
      }
    })();
    const measured = async (phase: string, work: () => Promise<unknown>) => {
      const began = performance.now();
      console.log("[activation-diagnostic]", JSON.stringify({phase, state: "start", ms: Math.round(began - start)}));
      try {
        const value = await work();
        console.log("[activation-diagnostic]", JSON.stringify({phase, state: "done", elapsedMs: Math.round(performance.now() - began)}));
        return value;
      } catch (error) {
        console.log("[activation-diagnostic]", JSON.stringify({phase, state: "failed", code: safeOutcome(error), elapsedMs: Math.round(performance.now() - began)}));
        throw error;
      }
    };
    try {
      await measured("onboarding", completeOnboarding);
      await waitForTauriBridge();
      await measured("initialize", initializeExtensionHost);
      await measured("host-ready", async () => {
        for (let attempt = 0; attempt < MAX_DIAGNOSTIC_SAMPLES; attempt += 1) {
          const host = await invokeTauri<{state: string}>("get_extension_host_status");
          if (host.state === "running") return;
          assert.notEqual(host.state, "error", "Host failed before activation");
          await browser.pause(DIAGNOSTIC_POLL_MS);
        }
        throw new Error("Host readiness deadline exceeded");
      });
      await measured("refresh", () => browser.refresh());
      await $('[data-e2e="app-root"]').waitForDisplayed();
      await measured("add", () => invokeTauri("add_local_extension", {path: FIXTURE}));
      await measured("enable", () => invokeTauri("set_extension_enabled", {extensionId: ID, enabled: true, trustConfirmed: true}));
      const records = await invokeTauri<Array<{manifest: {id: string}; status: string}>>("list_extensions");
      assert.equal(records.find((record) => record.manifest.id === ID)?.status, "active");
    } finally {
      sampling = false;
      await sampler;
      console.log("[activation-diagnostic]", JSON.stringify({phase: "end", elapsedMs: Math.round(performance.now() - start), samples}));
    }
  });
});
