import { setExtensionPresentation } from "./extension-presentation";
import assert from "node:assert/strict";
import en from "../../src/i18n/en.json";
import type { InstallJobView, InstallJobsSnapshot, InstallStatus } from "../../src/types/extension-install-jobs.generated";
import { invokeTauri } from "./tauri-invoke";

export async function snapshot(): Promise<InstallJobsSnapshot> {
  return invokeTauri("list_extension_installs");
}
export async function waitJob(id: string, status: InstallStatus): Promise<InstallJobView> {
  let job: InstallJobView | undefined;
  await browser.waitUntil(async () => {
    job = (await snapshot()).jobs.find(candidate => candidate.id === id);
    if (job?.status === "failed" && status !== "failed") assert.fail(`Installation failed: ${job.errorCode}`);
    return job?.status === status;
  }, { timeout: 20_000, timeoutMsg: `Installation did not reach ${status}` });
  assert.ok(job);
  return job;
}
export async function clickText(selector: string, text: string): Promise<void> {
  const clicked = await browser.execute((candidateSelector, candidateText) => {
    const candidate = Array.from(document.querySelectorAll<HTMLElement>(candidateSelector))
      .find(element => element.innerText.includes(candidateText));
    candidate?.click();
    return Boolean(candidate);
  }, selector, text);
  assert.equal(clicked, true, `Missing action: ${text}`);
}
export async function openExtensions(): Promise<void> {
  if (((await $(".app-root").getAttribute("class")) ?? "").includes("sidebar-hidden")) await toggleSidebar();
  await $(`button.lpf-btn[aria-label="${en.nav.settings}"]`).click();
  await clickText(".settings-subtab", en.settings.tabs.extensions);
  await clickText(".settings-tabbar-item", en.extensions.sections.custom);
}
export async function toggleSidebar(): Promise<void> {
  await browser.keys([process.platform === "darwin" ? "Meta" : "Control", "b", "NULL"]);
}
export async function openTracking(): Promise<void> {
  const button = $(".toolbar-btn-update");
  await button.waitForDisplayed();
  if (await button.getAttribute("aria-expanded") !== "true") await button.click();
  await $(".update-list").waitForDisplayed();
}
export async function closeTracking(): Promise<void> {
  await browser.keys("Escape");
  await $(".update-list").waitForExist({ reverse: true });
}
export async function present(theme: "light" | "dark"): Promise<void> {
  await setExtensionPresentation("en", theme);
}
