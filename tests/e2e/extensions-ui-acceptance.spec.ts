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
import { extensionThemeChoice } from "../../src/features/extension-ui/themes/theme-parser";
import {
  EXTENSION_HOST_SETUP_TIMEOUT_MS,
  EXTENSION_UI_SETUP_TIMEOUT_MS,
} from "../../scripts/e2e/extension-setup-deadline";
import { completeOnboarding } from "./onboarding-flow";
import { initializeExtensionHost } from "./extension-host-setup";
import { setMinimumViewport } from "./native-viewport";
import { invokeTauri, waitForTauriBridge } from "./tauri-invoke";

const FIXTURES = resolve("scripts/extensions/fixtures/ui");
const STANDARD_ID = "acceptance.standard.complete";
const THEME_ID = "acceptance.theme.valid";
const ADVANCED_ID = "acceptance.advanced.valid";
const LOCALES = ["fr", "en", "es", "de", "it", "zh", "ja"] as const;
const SETTINGS_LABELS = {
  de: deLocale.nav.settings,
  en: enLocale.nav.settings,
  es: esLocale.nav.settings,
  fr: frLocale.nav.settings,
  it: itLocale.nav.settings,
  ja: jaLocale.nav.settings,
  zh: zhLocale.nav.settings,
} as const;

interface CatalogEntry {
  extensionId: string;
  contributionId: string;
  contribution: { type: string; id: string; placement?: string };
}

interface CatalogSnapshot {
  contributions: CatalogEntry[];
  revision: number;
}

describe("extension UI installed acceptance", () => {
  before("initializes the extension host", async function () {
    this.timeout(EXTENSION_UI_SETUP_TIMEOUT_MS);
    await completeOnboarding();
    await waitForTauriBridge();
    await initializeExtensionHost();
    await waitForExtensionHost();
  });

  before("installs the standard extension fixture", async function () {
    this.timeout(EXTENSION_UI_SETUP_TIMEOUT_MS);
    const startup = await invokeTauri<{ mode: { kind: string } }>(
      "get_extension_ui_startup_state",
    );
    assert.equal(startup.mode.kind, "normal");
    // The E2E binary intentionally starts without external processes. Reload once
    // after the explicit host opt-in so every UI listener begins from a live host.
    await browser.refresh();
    await $('[data-e2e="app-root"]').waitForDisplayed();
    await install("standard-complete", STANDARD_ID);
    await waitForCatalog([STANDARD_ID]);
    await browser.pause(1_000);
    const recovery = await invokeTauri<{
      extensionId: string | null;
      stage: string | null;
      markerInvalid: boolean;
    }>("get_extension_recovery_state");
    assert.equal(recovery.extensionId, null, JSON.stringify(recovery));
  });

  before("installs the theme extension fixture", async function () {
    this.timeout(EXTENSION_UI_SETUP_TIMEOUT_MS);
    await install("theme-valid", THEME_ID);
    await waitForCatalog([STANDARD_ID, THEME_ID]);
  });

  before("installs the advanced extension fixture", async function () {
    this.timeout(EXTENSION_UI_SETUP_TIMEOUT_MS);
    await install("advanced-valid", ADVANCED_ID);
  });

  it("renders the installed standard, theme and advanced fixtures", async () => {
    const navigation = $('button.lpf-btn[aria-label="Acceptance"]');
    await navigation.waitForDisplayed();
    await navigation.click();
    await $(".xui-text").waitForDisplayed();
    assert.equal(await $(".xui-text").getText(), "Acceptance");

    const toolbar = $('button.xui-toolbar-action[aria-label="Acceptance"]');
    await toolbar.waitForDisplayed();
    await toolbar.click();
    await $(".toast-message").waitForDisplayed();
    assert.equal(await $(".toast-message").getText(), "Accepted");

    await $(".acceptance-advanced-button").waitForDisplayed();

    const catalog = await waitForCatalog([STANDARD_ID, THEME_ID]);
    assert.deepEqual(
      catalog.contributions
        .filter(({ extensionId }) => extensionId === STANDARD_ID)
        .map(({ contribution }) => contribution.placement)
        .sort(),
      [
        "agent.composer.leading",
        "app.navigation.primary",
        "app.toolbar.primary",
        "settings.navigation.preferences",
      ],
    );
    assert.equal(
      catalog.contributions.some(({ extensionId, contribution }) => (
        extensionId === THEME_ID && contribution.type === "theme"
      )),
      true,
    );

  });

  for (const locale of LOCALES) {
    for (const theme of RESOLVED_THEME_OPTIONS) {
      it(`keeps installed surfaces usable for ${locale}/${theme.id} at minimum size`, async () => {
        await setMinimumViewport(900, 600);
        await browser.execute((nextLocale, nextTheme) => {
          window.localStorage.setItem("clgo-language", nextLocale);
          window.localStorage.setItem("clgo-theme", nextTheme);
        }, locale, theme.id);
        await browser.refresh();
        await $('[data-e2e="app-root"]').waitForDisplayed();
        await browser.waitUntil(async () => (
          await browser.execute(() => document.documentElement.dataset.palette)
        ) === theme.id, { timeoutMsg: `Theme ${theme.id} was not applied` });
        await $(`button.lpf-btn[aria-label="${SETTINGS_LABELS[locale]}"]`).waitForDisplayed();
        await $('button.lpf-btn[aria-label="Acceptance"]').waitForDisplayed();
        await $(".acceptance-advanced-button").waitForDisplayed();
      });
    }
  }

  it("falls back from a removed extension theme and cleans every installed surface", async () => {
    const catalog = await waitForCatalog([STANDARD_ID, THEME_ID]);
    const theme = catalog.contributions.find(({ extensionId, contribution }) => (
      extensionId === THEME_ID && contribution.type === "theme"
    ));
    assert.ok(theme);
    const themeChoice = extensionThemeChoice(theme.extensionId, theme.contribution.id);
    await browser.execute((choice) => {
      window.localStorage.setItem("clgo-theme", choice);
    }, themeChoice);
    await browser.refresh();
    await $('[data-e2e="app-root"]').waitForDisplayed();
    await browser.waitUntil(async () => (
      await browser.execute(() => document.documentElement.dataset.palette)
    ) === theme.contribution.id, { timeoutMsg: "Extension theme was not applied" });

    await remove(THEME_ID);
    await browser.waitUntil(async () => (
      await browser.execute(() => window.localStorage.getItem("clgo-theme"))
    ) === "system", { timeoutMsg: "Removed theme did not fall back to system" });

    await remove(ADVANCED_ID);
    await $(".acceptance-advanced-button").waitForExist({ reverse: true });
    assert.equal(await browser.execute(
      () => document.documentElement.dataset.acceptanceAdvancedCleanup,
    ), "done");

    await remove(STANDARD_ID);
    await $('button.lpf-btn[aria-label="Acceptance"]').waitForExist({ reverse: true });
    await $('button.xui-toolbar-action[aria-label="Acceptance"]').waitForExist({ reverse: true });
  });
});

async function install(fixture: string, extensionId: string): Promise<void> {
  await invokeTauri("add_local_extension", { path: resolve(FIXTURES, fixture) });
  try {
    await invokeTauri("set_extension_enabled", {
      extensionId,
      enabled: true,
      trustConfirmed: true,
    });
  } catch (error) {
    const [recovery, host, extensions] = await Promise.all([
      invokeOrFallback<{
        extensionId: string | null;
        stage: string | null;
        markerInvalid: boolean;
      }>("get_extension_recovery_state", {
        extensionId: null,
        stage: null,
        markerInvalid: false,
      }),
      invokeOrFallback<{ state: string; lastError?: string }>(
        "get_extension_host_status",
        { state: "unavailable" },
      ),
      invokeOrFallback<Array<{ manifest: { id: string }; status: string; lastError?: string }>>(
        "list_extensions",
        [],
      ),
    ]);
    const record = extensions.find(({ manifest }) => manifest.id === extensionId);
    const reason = error instanceof Error ? error.message : "unknown";
    throw new Error(
      `Extension activation failed: extension=${extensionId}; reason=${reason}; stage=${recovery.stage ?? "none"}; markerInvalid=${recovery.markerInvalid}; host=${host.state}/${host.lastError ?? "none"}; record=${record?.status ?? "missing"}/${record?.lastError ?? "none"}`,
    );
  }
}

async function invokeOrFallback<T>(command: string, fallback: T): Promise<T> {
  try {
    return await invokeTauri<T>(command);
  } catch {
    return fallback;
  }
}

async function remove(extensionId: string): Promise<void> {
  await invokeTauri("remove_extension", { extensionId });
}

async function waitForCatalog(extensionIds: readonly string[]): Promise<CatalogSnapshot> {
  let snapshot: CatalogSnapshot = { revision: 0, contributions: [] };
  await browser.waitUntil(async () => {
    snapshot = await invokeTauri<CatalogSnapshot>("get_extension_ui_catalog");
    const identities = new Set(snapshot.contributions.map(({ extensionId }) => extensionId));
    return extensionIds.every((extensionId) => identities.has(extensionId));
  }, { timeoutMsg: "Installed extension UI catalog did not become ready" });
  return snapshot;
}

async function waitForExtensionHost(): Promise<void> {
  let latest: { state: string; lastError?: string; nodeVersion?: string } = {
    state: "unknown",
  };
  try {
    await browser.waitUntil(async () => {
      latest = await invokeTauri("get_extension_host_status");
      if (latest.state === "error") {
        throw new Error(`Extension host failed to start: ${latest.lastError ?? "unknown"}`);
      }
      return latest.state === "running";
    }, {
      timeout: EXTENSION_HOST_SETUP_TIMEOUT_MS,
      timeoutMsg: "Extension host did not become ready",
    });
  } catch {
    throw new Error(
      `Extension host unavailable: state=${latest.state}; code=${latest.lastError ?? "none"}; node=${latest.nodeVersion ?? "none"}`,
    );
  }
}
