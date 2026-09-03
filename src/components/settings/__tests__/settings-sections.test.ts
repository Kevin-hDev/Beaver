import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { SETTINGS_SECTIONS, SETTINGS_TAB_IDS } from "../settings-sections";
import type { CoreSettingsTabId } from "@/features/extension-ui/slot-types";

/* Le test fige la surface historique attendue ; la production la projette
   désormais depuis l'autorité unique des occupants cœur. */

const FR = JSON.parse(readFileSync("src/i18n/fr.json", "utf8")) as {
  settings: { sections: Record<string, string> };
};

const EXPECTED_CORE_TABS = [
  "general", "mascot", "shortcuts", "memory", "system-prompt", "tools", "advanced",
  "ollama", "forecast", "llm", "providers", "connectors", "channels", "extensions",
  "updates", "archived-chats", "about",
] as const satisfies readonly CoreSettingsTabId[];

describe("sections des Réglages", () => {
  it("range chaque onglet déclaré dans exactement une section", () => {
    expect(SETTINGS_TAB_IDS).toEqual(EXPECTED_CORE_TABS);
  });

  it("ne place aucun onglet dans deux sections", () => {
    const seen = new Set<string>();
    const duplicates = SETTINGS_TAB_IDS.filter((id) => {
      if (seen.has(id)) return true;
      seen.add(id);
      return false;
    });

    expect(duplicates).toEqual([]);
  });

  it("donne un titre traduit à chaque section", () => {
    const missing = SETTINGS_SECTIONS
      .map((section) => section.i18n)
      .filter((key) => {
        const name = key.replace("settings.sections.", "");
        return !(name in FR.settings.sections);
      });

    expect(missing).toEqual([]);
  });

  it("garde des sections courtes", () => {
    // Au-delà de quatre entrées, une section redevient la liste indifférenciée
    // qu'elle est censée découper.
    const tooLong = SETTINGS_SECTIONS
      .filter((section) => section.tabs.length > 4)
      .map((section) => section.i18n);

    expect(tooLong).toEqual([]);
  });
});
