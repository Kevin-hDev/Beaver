import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("voice probe translations", () => {
  it("contient tous les libellés dans les sept langues", () => {
    for (const locale of [fr, en, es, de, italian, zh, ja]) {
      expect(locale).toHaveProperty("voiceProbe.title");
      expect(locale).toHaveProperty("voiceProbe.start");
      expect(locale).toHaveProperty("voiceProbe.stop");
      expect(locale).toHaveProperty("voiceProbe.status.error");
      expect(locale).toHaveProperty("voiceProbe.samples");
      expect(locale).toHaveProperty("voiceProbe.level");
    }
  });
});
