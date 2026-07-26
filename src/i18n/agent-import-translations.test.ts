import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("agent import translations", () => {
  it("fournit les textes essentiels dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja];
    for (const locale of locales) {
      const section = locale.agentImport;
      expect(locale.onboarding.common.back.trim()).not.toBe("");
      expect(section.title.trim()).not.toBe("");
      expect(section.description.trim()).not.toBe("");
      expect(section.actions.all.trim()).not.toBe("");
      expect(section.actions.none.trim()).not.toBe("");
      expect(section.actions.confirmSource.trim()).not.toBe("");
      expect(section.conflict.replace.trim()).not.toBe("");
      expect(section.detail.partial.trim()).not.toBe("");
      expect(section.detail.updateAvailable.trim()).not.toBe("");
      expect(section.settings.manage.trim()).not.toBe("");
    }
  });
});
