import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itTranslations from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const languages: ReadonlyArray<Record<string, unknown>> = [
  de.agentLocal,
  en.agentLocal,
  es.agentLocal,
  fr.agentLocal,
  itTranslations.agentLocal,
  ja.agentLocal,
  zh.agentLocal,
];

describe("reasoning translations", () => {
  it("traduit Max et Ultra dans les sept langues", () => {
    for (const translations of languages) {
      expect(translations.reasoningMax).toBeTruthy();
      expect(translations.reasoningUltra).toBeTruthy();
    }
  });
});
