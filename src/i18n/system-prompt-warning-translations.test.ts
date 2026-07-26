import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

interface WarningLocale {
  ollama: {
    systemPromptWarningTitle: string;
    systemPromptWarningBody: string;
    systemPromptWarningRemember: string;
    systemPromptWarningContinue: string;
  };
}

describe("system prompt warning translations", () => {
  it("contient le message complet dans les sept langues sans nom de produit", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as WarningLocale[];
    for (const locale of locales) {
      const warning = locale.ollama;
      expect(warning.systemPromptWarningTitle).toBeTruthy();
      expect(warning.systemPromptWarningBody).toBeTruthy();
      expect(warning.systemPromptWarningRemember).toBeTruthy();
      expect(warning.systemPromptWarningContinue).toBeTruthy();
      expect(warning.systemPromptWarningBody).not.toMatch(/CL-GO/i);
    }
  });
});
