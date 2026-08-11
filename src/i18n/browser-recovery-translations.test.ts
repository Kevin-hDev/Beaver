import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itLocale from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("browser recovery translations", () => {
  it("traduit le message, le redémarrage et la fermeture dans les sept langues", () => {
    for (const locale of [de, en, es, fr, itLocale, ja, zh]) {
      expect(locale.browser.recoveryUnavailable).toBeTruthy();
      expect(locale.browser.restartApplication).toBeTruthy();
      expect(locale.browser.dismissRecovery).toBeTruthy();
    }
  });
});
