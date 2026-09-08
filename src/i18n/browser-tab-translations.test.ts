import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itLocale from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("browser tab translations", () => {
  it("décrit le déplacement au clavier dans les sept langues", () => {
    for (const locale of [de, en, es, fr, itLocale, ja, zh]) {
      const hint = locale.browser.reorderTabsHint;
      expect(hint).toContain("{{alt}}");
      expect(hint).toContain("Shift");
      expect(hint).toContain("Left");
      expect(hint).toContain("Right");
    }
  });
});
