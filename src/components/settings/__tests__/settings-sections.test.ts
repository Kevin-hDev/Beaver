import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { SETTINGS_SECTIONS, SETTINGS_TAB_IDS } from "../settings-sections";
import type { SettingsSubTab } from "@/types/navigation";

/* Les onglets ne sont plus déclarés dans une liste unique mais répartis en
   sections. Un onglet oublié lors de l'ajout d'une section n'apparaît nulle
   part dans la colonne, sans que rien ne le signale — d'où ces gardes. */

const NAV_TYPES = readFileSync("src/types/navigation.ts", "utf8");
const FR = JSON.parse(readFileSync("src/i18n/fr.json", "utf8")) as {
  settings: { sections: Record<string, string> };
};

function declaredSubTabs(): SettingsSubTab[] {
  const union = /export type SettingsSubTab\s*=\s*([^;]+);/.exec(NAV_TYPES)?.[1] ?? "";
  return [...union.matchAll(/"([^"]+)"/g)].map((match) => match[1] as SettingsSubTab);
}

describe("sections des Réglages", () => {
  it("range chaque onglet déclaré dans exactement une section", () => {
    const declared = declaredSubTabs();

    expect(declared.length).toBeGreaterThan(0);
    expect([...SETTINGS_TAB_IDS].sort()).toEqual([...declared].sort());
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
