import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";
import extensionContract from "../../src-tauri/resources/extension-host/contract.json";
import { EXTENSION_BACKEND_ERROR_CODES } from "@/lib/extension-errors";

const locales = [fr, en, es, de, itJson, zh, ja];
const diagnosticCodes = [
  ...extensionContract.diagnostics.hostCodes,
  ...extensionContract.diagnostics.runtimeCodes,
].sort();

function leafPaths(value: unknown, prefix = ""): string[] {
  if (!value || typeof value !== "object") return [prefix];
  return Object.entries(value)
    .flatMap(([key, child]) => leafPaths(child, prefix ? `${prefix}.${key}` : key))
    .sort();
}

describe("extensions translations", () => {
  it("fournit exactement les mêmes clés dans les sept langues", () => {
    const expected = leafPaths(en.extensions);

    for (const locale of locales) {
      expect(leafPaths(locale.extensions)).toEqual(expected);
      expect(locale.settings.tabs.extensions.trim()).not.toBe("");
      expect(locale.chatMenu.noPlugins.trim()).not.toBe("");
      expect(locale.extensions.fullAccessWarning.length).toBeGreaterThan(30);
      expect(locale.extensions.sensitiveAccessReminder.length).toBeGreaterThan(15);
    }
  });

  it("retire les anciens placeholders Plugins des Connecteurs et du chat", () => {
    for (const locale of locales) {
      expect("tabMcp" in locale.connectors.browse).toBe(false);
      expect("tabPlugins" in locale.connectors.browse).toBe(false);
      expect("pluginsEmpty" in locale.connectors.browse).toBe(false);
      expect("pluginsEmpty" in locale.chatMenu).toBe(false);
    }
  });

  it("partage la limite des plugins prioritaires avec l'interface", () => {
    for (const locale of locales) {
      expect(locale.extensions.discovery.count).toContain("{{max}}");
      expect(locale.extensions.discovery.count).not.toContain("/15");
    }
  });

  it("traduit chaque diagnostic d'extension dans les sept langues", () => {
    for (const locale of locales) {
      const translations = locale.extensions.diagnostics.codes as Record<string, string>;
      expect(Object.keys(translations).sort()).toEqual(diagnosticCodes);
      for (const translation of Object.values(translations)) {
        expect(translation.trim()).not.toBe("");
      }
    }
  });

  it("traduit chaque erreur du registre d'extensions dans les sept langues", () => {
    const expected = [...EXTENSION_BACKEND_ERROR_CODES].sort();
    for (const locale of locales) {
      const translations = locale.extensions.errors.codes as Record<string, string>;
      expect(Object.keys(translations).sort()).toEqual(expected);
      expect(Object.values(translations).every((value) => value.trim())).toBe(true);
    }
  });
});
