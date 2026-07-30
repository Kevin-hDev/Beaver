import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const locales = { de, en, es, fr, it: itJson, ja, zh };

type Locale = (typeof locales)[keyof typeof locales];

// Les ids sont lus dans les catalogues Rust, pas recopiés ici : un provider
// ajouté côté Rust sans ses traductions fait échouer ce test au lieu de sortir
// une carte sans texte en production.
const API_KEY_CATALOGS = [
  "../../src-tauri/src/services/llm/catalog.rs",
  "../../src-tauri/src/services/search/catalog.rs",
];

const FORECAST_CATALOGS = ["../../src-tauri/src/services/forecast/catalog_specs/providers.rs"];

interface ProviderCopy {
  description: string;
  freeTier: string;
}

function catalogProviderIds(relativePaths: string[]): string[] {
  const ids: string[] = [];
  for (const relativePath of relativePaths) {
    // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins constants déclarés en tête de fichier, aucune entrée externe
    const source = readFileSync(new URL(relativePath, import.meta.url), "utf8");
    for (const match of source.matchAll(/^\s+id: "([^"]+)",$/gmu)) {
      ids.push(match[1]);
    }
  }
  return ids;
}

// Deux sections et non une : l'id `google` désigne Gemini dans le catalogue LLM
// et TimesFM dans celui de prévision.
const SECTIONS = [
  {
    name: "apiKeys.providers",
    ids: catalogProviderIds(API_KEY_CATALOGS),
    pick: (locale: Locale): Record<string, ProviderCopy> => locale.apiKeys.providers,
  },
  {
    name: "forecast.providers",
    ids: catalogProviderIds(FORECAST_CATALOGS),
    pick: (locale: Locale): Record<string, ProviderCopy> => locale.forecast.providers,
  },
];

describe("api provider translations", () => {
  it("retrouve les providers des catalogues Rust", () => {
    const [apiKeys, forecast] = SECTIONS;
    expect(apiKeys.ids).toContain("groq");
    expect(apiKeys.ids).toContain("firecrawl");
    expect(apiKeys.ids.length).toBeGreaterThanOrEqual(13);
    expect(forecast.ids).toContain("nixtla");
    expect(forecast.ids.length).toBeGreaterThanOrEqual(10);
  });

  it("décrit chaque provider dans les sept langues", () => {
    for (const section of SECTIONS) {
      for (const [lang, locale] of Object.entries(locales)) {
        const providers = section.pick(locale);
        for (const id of section.ids) {
          const copy = providers[id];
          expect(copy, `${lang} → ${section.name}.${id} absent`).toBeDefined();
          expect(
            copy.description.trim(),
            `${lang} → ${section.name}.${id}.description vide`,
          ).not.toBe("");
          expect(copy.freeTier.trim(), `${lang} → ${section.name}.${id}.freeTier vide`).not.toBe(
            "",
          );
        }
      }
    }
  });

  it("ne garde pas de texte pour un provider sorti du catalogue", () => {
    for (const section of SECTIONS) {
      const known = new Set(section.ids);
      for (const [lang, locale] of Object.entries(locales)) {
        const orphans = Object.keys(section.pick(locale)).filter((id) => !known.has(id));
        expect(orphans, `${lang} garde des inconnus dans ${section.name}`).toEqual([]);
      }
    }
  });
});
