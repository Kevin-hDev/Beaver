import assert from "node:assert/strict";
import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { EXTENSION_UI_SETUP_TIMEOUT_MS } from "../../scripts/e2e/extension-setup-deadline";
import { completeOnboarding } from "./onboarding-flow";
import { initializeExtensionHost, waitForExtensionHost } from "./extension-host-setup";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

const STUCK_ID = "acceptance.cleanup.stuck";
const NEXT_ID = "acceptance.cleanup.next";

describe("advanced UI cleanup under a suspended extension callback", () => {
  it("removes the disabled extension and loads the following healthy UI", async function () {
    this.timeout(EXTENSION_UI_SETUP_TIMEOUT_MS);
    await completeOnboarding();
    await waitForTauriBridge();
    await initializeExtensionHost();
    await waitForExtensionHost();
    const root = await realpath(await mkdtemp(join(tmpdir(), "beaver-cleanup-proof-")));
    const installed: string[] = [];
    try {
      await install(root, STUCK_ID, true, installed);
      await $('[data-cleanup-proof="stuck"]').waitForExist();
      await invokeTauri("set_extension_enabled", {
        extensionId: STUCK_ID, enabled: false, trustConfirmed: false,
      });
      await browser.waitUntil(() => browser.execute(() =>
        document.documentElement.dataset.cleanupProofStarted === "yes"));
      await $('[data-cleanup-proof="stuck"]').waitForExist({ reverse: true });

      await install(root, NEXT_ID, false, installed);
      await $('[data-cleanup-proof="healthy"]').waitForExist();
      assert.equal(await $('[data-cleanup-proof="stuck"]').isExisting(), false);
    } finally {
      for (const extensionId of installed.reverse()) {
        await invokeTauri("remove_extension", { extensionId });
      }
      await rm(root, { recursive: true, force: true });
    }
  });
});

async function install(root: string, id: string, suspended: boolean, installed: string[]) {
  const directory = join(root, id);
  await mkdir(directory);
  await writeFile(join(directory, "beaver-extension.json"), JSON.stringify({
    id, name: "Cleanup proof", version: "1.0.0", beaverApi: "1", runtime: "node",
    main: "index.mjs", access: "full", apiLevel: "advanced", essential: false,
    ui: { apiVersion: "1", mode: "advanced", entry: "entry.mjs" },
  }));
  await writeFile(join(directory, "index.mjs"), "export default function activate() {}\n");
  await writeFile(join(directory, "entry.mjs"), `
export function activate(context) {
  context.mount("app.toolbar.primary", (container) => {
    container.dataset.cleanupProof = "${suspended ? "stuck" : "healthy"}";
  });
}
export function deactivate() {
  ${suspended ? 'document.documentElement.dataset.cleanupProofStarted = "yes"; return new Promise(() => {});' : "return;"}
}
`);
  await invokeTauri("add_local_extension", { path: directory });
  installed.push(id);
  await invokeTauri("set_extension_enabled", {
    extensionId: id, enabled: true, trustConfirmed: true,
  });
}
