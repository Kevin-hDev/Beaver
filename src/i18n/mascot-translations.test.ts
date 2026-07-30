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
      expect(locale.settings.mascot.circuitName.trim()).not.toBe("");
      expect(locale.settings.mascot.circuitDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.kovaName.trim()).not.toBe("");
      expect(locale.settings.mascot.kovaDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.nivalName.trim()).not.toBe("");
      expect(locale.settings.mascot.nivalDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.mokaiName.trim()).not.toBe("");
      expect(locale.settings.mascot.mokaiDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.voltName.trim()).not.toBe("");
      expect(locale.settings.mascot.voltDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.rakuName.trim()).not.toBe("");
      expect(locale.settings.mascot.rakuDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.picoName.trim()).not.toBe("");
      expect(locale.settings.mascot.picoDesc.trim()).not.toBe("");
      expect(locale.settings.mascot.selected.trim()).not.toBe("");
      expect(locale.settings.mascot.moveLabel.trim()).not.toBe("");
    }
  });
});
