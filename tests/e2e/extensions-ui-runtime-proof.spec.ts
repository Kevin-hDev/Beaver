import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("extension UI runtime protocol", () => {
  it("imports the allowlisted local ESM fixture under the final CSP", async () => {
    const bytes = readFileSync(resolve("src-tauri/tests/fixtures/extensions/ui-advanced/entry.mjs"));
    const manifestSha = createHash("sha256").update(bytes).digest("hex");
    const origin = process.platform === "win32"
      ? "http://beaver-extension.localhost"
      : "beaver-extension://localhost";
    const url = `${origin}/ui-proof/${manifestSha}/entry.mjs`;

    const result = await browser.executeAsync((moduleUrl, done) => {
      import(moduleUrl).then(() => {
        done(document.documentElement.dataset.extensionUiRuntimeProof);
      }).catch(() => done("failed"));
    }, url);

    assert.equal(result, "loaded");
  });
});
