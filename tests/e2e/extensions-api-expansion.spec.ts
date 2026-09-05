import assert from "node:assert/strict";
import { resolve } from "node:path";
import deLocale from "../../src/i18n/de.json";
import enLocale from "../../src/i18n/en.json";
import esLocale from "../../src/i18n/es.json";
import frLocale from "../../src/i18n/fr.json";
import itLocale from "../../src/i18n/it.json";
import jaLocale from "../../src/i18n/ja.json";
import zhLocale from "../../src/i18n/zh.json";
import { RESOLVED_THEME_OPTIONS } from "../../src/lib/app-themes";
import { TIMEOUTS } from "../../src/types/extension-contract.generated";
import { completeOnboarding } from "./onboarding-flow";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

const extensionId = "acceptance.api.expansion";
const fixture = process.env.BEAVER_E2E_API_EXPANSION_FIXTURE
  ?? resolve("src-tauri/tests/fixtures/extensions/api-expansion");
const LOCALES = {
  de: deLocale,
  en: enLocale,
  es: esLocale,
  fr: frLocale,
  it: itLocale,
  ja: jaLocale,
  zh: zhLocale,
} as const;

type LocaleCopy = typeof enLocale;

interface ExtensionView {
  enabled: boolean;
  manifest: { id: string; name: string };
  contributions: { tools: unknown[]; skills: unknown[]; resources: unknown[] };
}

describe("API expansion packaged acceptance", () => {
  let installed = false;

  before(async function () {
    this.timeout(TIMEOUTS.hostRequestTimeoutMs + TIMEOUTS.hostStopTimeoutMs);
    await completeOnboarding();
    await waitForTauriBridge();
    await invokeTauri("e2e_initialize_extension_host");
    await waitForExtensionHost();
    await invokeTauri("add_local_extension", { path: fixture });
    installed = true;
    await invokeTauri("set_extension_enabled", {
      extensionId,
      enabled: true,
      trustConfirmed: true,
    });
  });

  after(async () => {
    if (!installed) return;
    await invokeTauri("set_permission_mode", { mode: "auto" });
    await invokeTauri("remove_extension", { extensionId });
  });

  it("loads Beryl with its two tools, skill and two resources", async () => {
    const extension = await view();
    assert.equal(extension.manifest.name, "Beryl");
    assert.equal(extension.enabled, true);
    assert.equal(extension.contributions.tools.length, 2);
    assert.equal(extension.contributions.skills.length, 1);
    assert.equal(extension.contributions.resources.length, 2);
  });

  it("shows the complete capability unit with accessible controls", async () => {
    await setPresentation("en", "light");
    await openExtensionDetail(enLocale);
    assert.deepEqual(await textList(".extd-tool-row code"), [
      `${extensionId}.catalog_probe`,
      `${extensionId}.produce_artifacts`,
    ]);
    assert.deepEqual(await textList(".extcap-row-heading > span:first-child"), [
      "reference-skill",
      "reference",
      "preview",
    ]);
    const capabilities = $(".extcap-root");
    const labelledBy = await capabilities.getAttribute("aria-labelledby");
    assert.ok(labelledBy);
    assert.equal(await browser.execute((id) => Boolean(document.getElementById(id)), labelledBy), true);
    assert.equal(await browser.execute(
      () => document.querySelectorAll('button[aria-label*="Beryl"]').length > 0,
    ), true);
  });

  for (const [locale, copy] of Object.entries(LOCALES)) {
    it(`keeps capabilities readable in ${locale}`, async () => {
      await setPresentation(locale, "light");
      await openExtensionDetail(copy);
      assert.equal(await $(".extcap-root h3").getText(), copy.extensions.detail.capabilities);
      assert.deepEqual(await textList(".extcap-group h4"), [
        copy.extensions.detail.skills,
        copy.extensions.detail.resources,
      ]);
    });
  }

  for (const theme of RESOLVED_THEME_OPTIONS) {
    it(`keeps the capability detail usable at narrow width in ${theme.id}`, async () => {
      await browser.setWindowSize(900, 600);
      await setPresentation("en", theme.id);
      await openExtensionDetail(enLocale);
      assert.equal(
        await browser.execute(() => document.documentElement.dataset.palette),
        theme.id,
      );
      await $(".extcap-root").waitForDisplayed();
    });
  }

  it("opens the extension detail with the keyboard", async () => {
    await setPresentation("en", "dark");
    await openExtensionsPage(enLocale);
    await waitForText(".extr-main", "Beryl");
    await browser.execute(() => {
      Array.from(document.querySelectorAll<HTMLElement>(".extr-main"))
        .find((row) => row.innerText.includes("Beryl"))?.focus();
    });
    await browser.keys("Enter");
    await $(".settings-detail-title h2").waitForDisplayed();
    assert.equal(await $(".settings-detail-title h2").getText(), "Beryl");
  });

  it("keeps extension UI and shortcuts out of Chat mode", async () => {
    await invokeTauri("set_extension_show_in_chat", { extensionId, showInChat: true });
    await invokeTauri("set_permission_mode", { mode: "chat" });
    await setPresentation("en", "light");
    const sessionButton = $(`button.lpf-btn[aria-label="${enLocale.nav.agentLocal}"]`);
    await sessionButton.waitForDisplayed();
    await sessionButton.click();
    await $(".perm-mode-trigger-chat").waitForDisplayed();
    await $(".chat-plus-btn").click();
    await $(".cpm-dropdown").waitForDisplayed();
    assert.equal((await textList(".cpm-item")).includes(enLocale.chatMenu.plugins), false);
    assert.equal((await $("body").getText()).includes("Beryl"), false);
    await invokeTauri("set_permission_mode", { mode: "auto" });
    await invokeTauri("set_extension_show_in_chat", { extensionId, showInChat: false });
  });

  it("keeps registered contributions after an explicit host reload", async () => {
    await invokeTauri("reload_extension_host");
    await waitForExtensionHost();
    const extension = await view();
    assert.equal(extension.contributions.tools.length, 2);
    assert.equal(extension.contributions.skills.length, 1);
    assert.equal(extension.contributions.resources.length, 2);
  });

  it("removes the extension from active use when disabled", async () => {
    await invokeTauri("set_extension_enabled", {
      extensionId,
      enabled: false,
      trustConfirmed: false,
    });
    assert.equal((await view()).enabled, false);
  });
});

async function setPresentation(locale: string, theme: string): Promise<void> {
  await browser.execute((nextLocale, nextTheme) => {
    window.localStorage.setItem("clgo-language", nextLocale);
    window.localStorage.setItem("clgo-theme", nextTheme);
  }, locale, theme);
  await browser.refresh();
  await $('[data-e2e="app-root"]').waitForDisplayed();
  await browser.waitUntil(async () => (
    await browser.execute(() => document.documentElement.dataset.palette)
  ) === theme, { timeoutMsg: `Theme ${theme} was not applied` });
}

async function openExtensionsPage(copy: LocaleCopy): Promise<void> {
  const settings = $(`button.lpf-btn[aria-label="${copy.nav.settings}"]`);
  await settings.waitForDisplayed();
  await settings.click();
  await clickWithText(".settings-subtab", copy.settings.tabs.extensions);
  await clickWithText(".settings-tabbar-item", copy.extensions.sections.custom);
  await waitForText(".extr-main", "Beryl");
}

async function openExtensionDetail(copy: LocaleCopy): Promise<void> {
  await openExtensionsPage(copy);
  await clickWithText(".extr-main", "Beryl");
  await $(".settings-detail-title h2").waitForDisplayed();
}

async function waitForText(selector: string, text: string): Promise<void> {
  await browser.waitUntil(() => browser.execute(
    (candidateSelector, candidateText) => Array.from(
      document.querySelectorAll<HTMLElement>(candidateSelector),
    ).some((element) => element.innerText.includes(candidateText)),
    selector,
    text,
  ), { timeoutMsg: `Missing ${selector} containing ${text}` });
}

async function clickWithText(selector: string, text: string): Promise<void> {
  await waitForText(selector, text);
  const clicked = await browser.execute((candidateSelector, candidateText) => {
    const element = Array.from(document.querySelectorAll<HTMLElement>(candidateSelector))
      .find((candidate) => candidate.innerText.includes(candidateText));
    element?.click();
    return Boolean(element);
  }, selector, text);
  assert.equal(clicked, true);
}

async function textList(selector: string): Promise<string[]> {
  return browser.execute(
    (candidateSelector) => Array.from(document.querySelectorAll<HTMLElement>(candidateSelector))
      .map((element) => element.innerText),
    selector,
  );
}

async function view(): Promise<ExtensionView> {
  const extensions = await invokeTauri<ExtensionView[]>("list_extensions");
  const extension = extensions.find(({ manifest }) => manifest.id === extensionId);
  assert.ok(extension, "API expansion fixture is missing from the extension registry");
  return extension;
}

async function waitForExtensionHost(): Promise<void> {
  let latest = { state: "unknown", lastError: undefined as string | undefined };
  await browser.waitUntil(async () => {
    latest = await invokeTauri("get_extension_host_status");
    if (latest.state === "error") throw new Error(latest.lastError ?? "host_error");
    return latest.state === "running";
  }, {
    timeout: TIMEOUTS.hostRequestTimeoutMs + TIMEOUTS.hostStopTimeoutMs,
    timeoutMsg: `Extension host unavailable: ${latest.state}`,
  });
}
