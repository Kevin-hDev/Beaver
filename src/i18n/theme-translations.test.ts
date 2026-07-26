import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

interface ThemeLocale {
  settings: {
    emeraldNight: string;
    cobaltFrost: string;
    astralMist: string;
    crimsonEclipse: string;
  };
}

describe("theme translations", () => {
  it("nomme les thèmes personnalisés dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as ThemeLocale[];

    for (const locale of locales) {
      expect(locale.settings.emeraldNight.trim()).not.toBe("");
      expect(locale.settings.cobaltFrost.trim()).not.toBe("");
      expect(locale.settings.astralMist.trim()).not.toBe("");
      expect(locale.settings.crimsonEclipse.trim()).not.toBe("");
    }
  });
});
