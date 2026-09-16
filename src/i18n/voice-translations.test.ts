import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

function keys(value: unknown, prefix = ""): string[] {
  if (!value || typeof value !== "object" || Array.isArray(value)) return [prefix];
  return Object.entries(value).flatMap(([key, child]) => keys(child, prefix ? `${prefix}.${key}` : key));
}

describe("voice translations", () => {
  it("has the same voice keys in all seven locales", () => {
    const expected = keys(en.voice).sort();
    for (const locale of [fr, es, de, italian, zh, ja]) expect(keys(locale.voice).sort()).toEqual(expected);
  });
});
