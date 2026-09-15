import { describe, expect, it } from "vitest";
import { voiceLanguageOptions } from "../voice-language-options";

describe("voiceLanguageOptions", () => {
  it("deduplicates and localizes catalogue codes", () => {
    const options = voiceLanguageOptions(["fr", "en", "fr"], "fr");
    expect(options).toHaveLength(2);
    expect(options.map((option) => option.value).sort()).toEqual(["en", "fr"]);
    expect(options.find((option) => option.value === "fr")?.label).toBe("français");
  });
});
