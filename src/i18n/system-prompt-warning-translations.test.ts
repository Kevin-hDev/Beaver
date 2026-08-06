import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

interface WarningLocale {
  settings: {
    tabs: { systemPrompt: string };
    systemPrompt: {
      title: string;
      useBeaver: string;
      restoreOllama: string;
      restoreDefault: string;
      warning: {
        title: string;
        global: { body: string };
        ollama: { body: string };
        remember: string;
        continue: string;
      };
    };
  };
}

describe("system prompt warning translations", () => {
  it("contient les deux avertissements distincts dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as WarningLocale[];
    for (const locale of locales) {
      const { warning } = locale.settings.systemPrompt;
      expect(locale.settings.tabs.systemPrompt).toBeTruthy();
      expect(locale.settings.systemPrompt.title).toBeTruthy();
      expect(locale.settings.systemPrompt.useBeaver).toBeTruthy();
      expect(locale.settings.systemPrompt.restoreOllama).toBeTruthy();
      expect(locale.settings.systemPrompt.restoreDefault).toBeTruthy();
      expect(warning.title).toBeTruthy();
      expect(warning.global.body).toBeTruthy();
      expect(warning.ollama.body).toBeTruthy();
      expect(warning.global.body).not.toBe(warning.ollama.body);
      expect(warning.remember).toBeTruthy();
      expect(warning.continue).toBeTruthy();
    }
  });
});
