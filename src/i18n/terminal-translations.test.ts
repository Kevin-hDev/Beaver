import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const translations = { de, en, es, fr, it: italian, ja, zh };
const keys = [
  "tabsLoadFailed",
  "tabsSaveFailed",
  "tabLimitReached",
  "liveLimitReached",
  "failedToClose",
  "inputQueueFull",
  "inputFailed",
] as const;

describe("traductions de la persistance terminal", () => {
  it.each(Object.entries(translations))("définit les sept messages en %s", (_locale, value) => {
    const terminal = value.terminal as Record<string, string>;
    for (const key of keys) {
      const message = terminal[key];
      expect(typeof message).toBe("string");
      expect(message?.trim()).not.toBe("");
    }
  });
});
