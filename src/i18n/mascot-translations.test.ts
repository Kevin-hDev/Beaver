import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itTranslations from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("mascot translations", () => {
  it("couvre l'onglet et ses réglages dans les sept langues", () => {
    for (const locale of [fr, en, es, de, itTranslations, zh, ja]) {
      expect(locale.settings.tabs.mascot.trim()).not.toBe("");
      expect(locale.settings.mascot.enabledTitle.trim()).not.toBe("");
      expect(locale.settings.mascot.sizeTitle.trim()).not.toBe("");
      expect(locale.settings.mascot.moveLabel.trim()).not.toBe("");
    }
  });
});
