import { describe, expect, it } from "vitest";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itCatalog from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";
import { KNOWN_ERROR_KEYS } from "./agent-error-codes";

const catalogs: ReadonlyArray<Record<string, unknown>> = [fr, en, es, de, itCatalog, zh, ja];

describe("KNOWN_ERROR_KEYS", () => {
  it("pointe vers un message traduit dans les sept langues", () => {
    for (const translationKey of Object.values(KNOWN_ERROR_KEYS)) {
      for (const catalog of catalogs) {
        expect(readTranslation(catalog, translationKey)).not.toBeUndefined();
      }
    }
  });
});

function readTranslation(catalog: Record<string, unknown>, path: string): unknown {
  return path.split(".").reduce<unknown>((value, segment) => {
    if (!value || typeof value !== "object") return undefined;
    return (value as Record<string, unknown>)[segment];
  }, catalog);
}
