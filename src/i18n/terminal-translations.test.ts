import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const translations = { de, en, es, fr, it: italian, ja, zh };
const keys = ["tabsLoadFailed", "tabsSaveFailed", "tabLimitReached"] as const;

describe("traductions de la persistance terminal", () => {
  it.each(Object.entries(translations))("définit les trois messages en %s", (_locale, value) => {
    for (const key of keys) {
      expect(value.terminal[key].trim()).not.toBe("");
    }
  });
});
