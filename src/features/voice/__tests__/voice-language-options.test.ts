import { describe, expect, it } from "vitest";
import { resolveVoiceLanguage, voiceLanguageChoices, voiceLanguageOptions } from "../voice-language-options";

describe("voiceLanguageOptions", () => {
  it("deduplicates and localizes catalogue codes", () => {
    const options = voiceLanguageOptions(["fr", "en", "fr"], "fr");
    expect(options).toHaveLength(2);
    expect(options.map((option) => option.value).sort()).toEqual(["en", "fr"]);
    expect(options.find((option) => option.value === "fr")?.label).toBe("français");
  });

  it("resolves the interface language and gives Cohere a required fallback", () => {
    expect(resolveVoiceLanguage({ kind: "follow-interface" }, "fr-FR", "explicit-only"))
      .toEqual({ kind: "language", value: "fr" });
    expect(resolveVoiceLanguage({ kind: "automatic" }, "de", "explicit-only"))
      .toEqual({ kind: "language", value: "de" });
    expect(resolveVoiceLanguage({ kind: "automatic" }, "fr", "automatic-only"))
      .toEqual({ kind: "automatic" });
    expect(resolveVoiceLanguage({ kind: "language", value: "fr" }, "fr", "automatic-only"))
      .toEqual({ kind: "automatic" });
  });

  it("offers only choices the selected engine can apply", () => {
    const labels = { automatic: "Détection automatique", followInterface: "Suivre l’interface" };
    expect(voiceLanguageChoices(["fr", "en"], "fr", "automatic-only", labels))
      .toEqual([{ value: "automatic", label: "Détection automatique" }]);
    expect(voiceLanguageChoices(["fr", "en"], "fr", "explicit-only", labels).map(({ value }) => value))
      .toEqual(["follow-interface", "en", "fr"]);
  });
});
