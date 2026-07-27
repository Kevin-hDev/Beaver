import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const locales = [fr, en, es, de, itJson, zh, ja];

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
});
