import { describe, expect, it } from "vitest";

import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const locales = { de, en, es, fr, it: itJson, ja, zh };

describe("update translations", () => {
  it("décrit les erreurs de préparation et d'installation dans les sept langues", () => {
    for (const [language, locale] of Object.entries(locales)) {
      expect(locale.errors.updatePrepareFailed, language).toBeTruthy();
      expect(locale.errors.updateInstallFailed, language).toBeTruthy();
      expect(locale.updates.dismiss, language).toBeTruthy();
      expect(locale.updates.cancelled, language).toBeTruthy();
      expect(locale.settings.tabs.updates, language).toBeTruthy();
      expect(locale.settings.updates.availableTitle, language).toBeTruthy();
    }
  });
});
