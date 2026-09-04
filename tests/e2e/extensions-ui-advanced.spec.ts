import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("advanced extension UI lifecycle", () => {
  it("imports, mounts and cleans the allowlisted local fixture", async () => {
    const bytes = readFileSync(resolve("src-tauri/tests/fixtures/extensions/ui-advanced/entry.mjs"));
    const manifestSha = createHash("sha256").update(bytes).digest("hex");
    const origin = process.platform === "win32"
      ? "http://beaver-extension.localhost"
      : "beaver-extension://localhost";
    const url = `${origin}/ui-proof/${manifestSha}/entry.mjs`;

    const result = await browser.executeAsync((moduleUrl, done) => {
      const anchor = document.createElement("span");
      document.body.append(anchor);
      import(moduleUrl).then(async (loaded: unknown) => {
        if (!loaded || typeof loaded !== "object"
          || typeof (loaded as { activate?: unknown }).activate !== "function") {
          throw new Error("invalid_fixture");
        }
        const module = loaded as {
          activate: (context: {
            apiVersion: string;
            extensionId: string;
            mount: (placement: string, mount: (container: HTMLElement) => unknown) => void;
            completeWithoutMounts: () => void;
          }) => unknown;
          deactivate?: () => unknown;
        };
        let mountCleanup: unknown;
        const activationCleanup = await module.activate({
          apiVersion: "1",
          extensionId: "ui-proof",
          mount: (_placement, mount) => { mountCleanup = mount(anchor); },
          completeWithoutMounts: () => {},
        });
        const mounted = anchor.dataset.extensionUiFixture;
        if (typeof activationCleanup === "function") {
          await (activationCleanup as () => unknown)();
        }
        if (typeof mountCleanup === "function") {
          await (mountCleanup as () => unknown)();
        }
        if (typeof module.deactivate === "function") await module.deactivate();
        done({
          mounted,
          cleaned: document.documentElement.dataset.extensionUiFixtureCleanup,
          deactivated: document.documentElement.dataset.extensionUiFixtureDeactivate,
        });
      }).catch(() => done({ mounted: "failed" }));
    }, url);

    assert.deepEqual(result, {
      mounted: "mounted",
      cleaned: "done",
      deactivated: "done",
    });
  });
});
