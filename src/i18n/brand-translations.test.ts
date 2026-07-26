import { describe, expect, it } from "vitest";

import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const locales = { de, en, es, fr, it: itJson, ja, zh };
const LEGACY_PUBLIC_NAME = /CL[-]GO(?:[-]DASH)?|\/cl[-]go\b/iu;
const MAX_TRANSLATION_VALUES = 10_000;

function stringValues(value: unknown, output: string[] = []): string[] {
  if (output.length > MAX_TRANSLATION_VALUES) {
    throw new Error("Translation catalog is too large");
  }
  if (typeof value === "string") {
    output.push(value);
    return output;
  }
  if (Array.isArray(value)) {
    for (const item of value) stringValues(item, output);
    return output;
  }
  if (value && typeof value === "object") {
    for (const item of Object.values(value)) stringValues(item, output);
  }
  return output;
}

describe("Beaver translations", () => {
  it("removes the legacy public name from all seven locales", () => {
    for (const [locale, catalog] of Object.entries(locales)) {
      const legacyValues = stringValues(catalog).filter((value) =>
        LEGACY_PUBLIC_NAME.test(value),
      );

      expect(legacyValues, locale).toEqual([]);
    }
  });
});
