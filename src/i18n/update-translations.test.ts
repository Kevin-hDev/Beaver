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
      expect(locale.installer.windowTitle, language).toBeTruthy();
      expect(locale.installer.cancelledBody, language).toBeTruthy();
      expect(locale.updates.window.restarting, language).toBeTruthy();
    }
  });

  it("garde les contrats installateur et fenêtre de mise à jour identiques", () => {
    const keys = (value: object, prefix = ""): string[] =>
      Object.entries(value).flatMap(([key, child]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        return child && typeof child === "object" ? keys(child as object, path) : [path];
      });
    const expected = keys({ installer: en.installer, window: en.updates.window }).sort();
    for (const [language, locale] of Object.entries(locales)) {
      expect(
        keys({ installer: locale.installer, window: locale.updates.window }).sort(),
        language,
      ).toEqual(expected);
    }
  });
});
