import assert from "node:assert/strict";
import { join } from "node:path";
import en from "../../src/i18n/en.json";
import type { InstallJobView } from "../../src/types/extension-install-jobs.generated";
import { EXTENSION_UI_SETUP_TIMEOUT_MS } from "../../scripts/e2e/extension-setup-deadline";
import { completeOnboarding } from "./onboarding-flow";
import { initializeExtensionHost, waitForExtensionHost } from "./extension-host-setup";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";
import { setMinimumViewport } from "./native-viewport";
import { clickText, closeTracking, openExtensions, openTracking, present, snapshot, toggleSidebar, waitJob } from "./extensions-install-background.support";

interface Fixture { npm: string; git: string; local: string }
const copy = en.extensionInstalls;

describe("background extension installation", () => {
  let fixture: Fixture;
  let baselineJobs = new Set<string>();
  before(async function () {
    this.timeout(EXTENSION_UI_SETUP_TIMEOUT_MS);
    await completeOnboarding();
    await waitForTauriBridge();
    await initializeExtensionHost();
    await waitForExtensionHost();
    const capability = await invokeTauri<{ status: string }>("browser_capability");
    console.info("CEF status before installation journey:", capability.status);
    baselineJobs = new Set((await snapshot()).jobs.map(job => job.id));
    fixture = await invokeTauri<Fixture>("e2e_extension_install_fixture", { enabled: true });
    await present("light");
    await setMinimumViewport();
  });
  after(async () => {
    for (const job of (await snapshot()).jobs.filter(candidate => !baselineJobs.has(candidate.id))) {
      if (job.canCancel) { await invokeTauri("cancel_extension_install", { jobId: job.id }); await waitJob(job.id, "cancelled"); }
      if (job.extensionId) await invokeTauri("remove_extension", { extensionId: job.extensionId });
      await invokeTauri("dismiss_extension_install", { jobId: job.id });
    }
    await invokeTauri("e2e_extension_install_fixture", { enabled: false });
    await present("light");
  });

  it("admits from the add dialog, navigates away and exposes the stopped request with sidebar collapsed", async () => {
    await openExtensions();
    await clickText(".settings-panel-header button", en.extensions.actions.add);
    await clickText(".exta-option", en.extensions.add.npm);
    await $("#exta-source-input").setValue(fixture.npm);
    await $(".exta-source-form button[type=submit]").click();
    await $(".exta-dialog").waitForExist({ reverse: true });
    const job = (await snapshot()).jobs.find(candidate => candidate.kind === "npm" && candidate.displayName === fixture.npm);
    assert.ok(job);
    await waitJob(job.id, "awaitingConfirmation");
    // Navigation is a normal UI action while the backend still awaits consent.
    await $(`button.lpf-btn[aria-label="${en.nav.agentLocal}"]`).click();
    await toggleSidebar();
    await browser.waitUntil(async () => ((await $(".app-root").getAttribute("class")) ?? "").includes("sidebar-hidden"));
    await openTracking();
    const row = $(`.update-list [data-install-job="${job.id}"]`);
    assert.ok((await row.getText()).includes(copy.status.awaitingConfirmation));
    assert.equal(await row.$('[role="progressbar"]').isExisting(), false);
    await closeTracking();
    assert.equal((await waitJob(job.id, "awaitingConfirmation")).canCancel, true);

    const queued = await invokeTauri<InstallJobView>("start_extension_install", { request: { kind: "npm", locator: `${fixture.npm}-queued` } });
    await openTracking();
    const queuedRow = $(`.update-list [data-install-job="${queued.id}"]`);
    await queuedRow.waitForDisplayed();
    assert.ok((await queuedRow.getText()).includes(fixture.npm));
    await clickText(`.update-list [data-install-job="${queued.id}"] button`, copy.showRequest);
    assert.equal(await browser.execute(id => document.activeElement?.getAttribute("data-install-job") === id, job.id), true);
    await clickText(`.update-list [data-install-job="${queued.id}"] button`, copy.cancel);
    await waitJob(queued.id, "cancelled");
    await clickText(`.update-list [data-install-job="${job.id}"] button`, copy.continue);
    const completed = await waitJob(job.id, "completed");
    assert.equal(await $(".exta-dialog").isExisting(), false);
    assert.equal(await $(".extd-root").isExisting(), false);
    assert.ok(completed.extensionId);
    const records = await invokeTauri<Array<{ manifest: { id: string }; trusted: boolean; enabled: boolean }>>("list_extensions");
    const installed = records.find(record => record.manifest.id === completed.extensionId);
    assert.ok(installed);
    assert.equal(installed.trusted, false);
    assert.equal(installed.enabled, false);
    await closeTracking();
  });

  it("keeps pending work across a frontend reload and allows real cancellation in both themes", async () => {
    for (const theme of ["light", "dark"] as const) {
      const job = await invokeTauri<InstallJobView>("start_extension_install", { request: { kind: "npm", locator: `${fixture.npm}-${theme}` } });
      await waitJob(job.id, "awaitingConfirmation");
      await present(theme);
      await openTracking();
      const row = $(`.update-list [data-install-job="${job.id}"]`);
      await row.waitForDisplayed();
      assert.ok((await row.getText()).includes(copy.status.awaitingConfirmation));
      const artifact = process.env.E2E_ARTIFACT_DIR;
      if (artifact) await browser.saveScreenshot(join(artifact, `extension-install-${theme}.png`));
      if (theme === "dark") {
        await browser.setWindowSize(700, 640);
        await browser.waitUntil(async () => browser.execute(() => {
          const panel = document.querySelector(".update-list")?.getBoundingClientRect();
          return Boolean(panel && panel.left >= 0 && panel.right <= innerWidth);
        }));
        if (artifact) await browser.saveScreenshot(join(artifact, "extension-install-narrow.png"));
      }
      await clickText(`.update-list [data-install-job="${job.id}"] button`, copy.cancel);
      await waitJob(job.id, "cancelled");
      await closeTracking();
      await setMinimumViewport();
    }
  });

  it("installs the local Git fixture with explicit volume consent", async () => {
    const job = await invokeTauri<InstallJobView>("start_extension_install", { request: { kind: "git", locator: fixture.git } });
    await waitJob(job.id, "awaitingConfirmation");
    await openTracking();
    await clickText(`.update-list [data-install-job="${job.id}"] button`, copy.continue);
    await waitJob(job.id, "completed");
    await closeTracking();
  });

  it("admits a local source through the same tracker without granting trust", async () => {
    const git = (await snapshot()).jobs.find(job => job.kind === "git" && job.extensionId);
    assert.ok(git?.extensionId);
    await invokeTauri("remove_extension", { extensionId: git.extensionId });
    await invokeTauri("dismiss_extension_install", { jobId: git.id });
    const job = await invokeTauri<InstallJobView>("start_extension_install", {
      request: { kind: "local", path: fixture.local },
    });
    const completed = await waitJob(job.id, "completed");
    assert.ok(completed.extensionId);
    await openTracking();
    await $(`.update-list [data-install-job="${job.id}"]`).waitForDisplayed();
    await closeTracking();
  });
});
