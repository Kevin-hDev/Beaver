import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

interface DirectoryAccessLocale {
  directoryAccess: {
    title: string;
    description: string;
    help: string;
    settings: string;
    error: string;
  };
}

describe("directory access translations", () => {
  it("contient le message et les actions dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as DirectoryAccessLocale[];

    for (const locale of locales) {
      expect(Object.values(locale.directoryAccess).every((value) => value.trim())).toBe(true);
    }
  });
});
