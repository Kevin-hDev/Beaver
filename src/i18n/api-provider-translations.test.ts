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

// Les ids sont lus dans les catalogues Rust, pas recopiés ici : un provider
// ajouté côté Rust sans ses traductions fait échouer ce test au lieu de sortir
// une carte sans texte en production.
const CATALOGS = [
  "../../src-tauri/src/services/llm/catalog.rs",
  "../../src-tauri/src/services/search/catalog.rs",
];

interface ProviderCopy {
  description: string;
  freeTier: string;
}

function catalogProviderIds(): string[] {
  const ids: string[] = [];
  for (const relativePath of CATALOGS) {
    // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins constants déclarés dans CATALOGS, aucune entrée externe
    const source = readFileSync(new URL(relativePath, import.meta.url), "utf8");
    for (const match of source.matchAll(/^\s+id: "([^"]+)",$/gmu)) {
      ids.push(match[1]);
    }
  }
  return ids;
}

describe("api provider translations", () => {
  const providerIds = catalogProviderIds();

  it("retrouve les providers des deux catalogues Rust", () => {
    expect(providerIds).toContain("groq");
    expect(providerIds).toContain("firecrawl");
    expect(providerIds.length).toBeGreaterThanOrEqual(13);
  });

  it("décrit chaque provider dans les sept langues", () => {
    for (const [lang, locale] of Object.entries(locales)) {
      const providers: Record<string, ProviderCopy> = locale.apiKeys.providers;
      for (const id of providerIds) {
        const copy = providers[id];
        expect(copy, `${lang} → ${id} absent`).toBeDefined();
        expect(copy.description.trim(), `${lang} → ${id}.description vide`).not.toBe("");
        expect(copy.freeTier.trim(), `${lang} → ${id}.freeTier vide`).not.toBe("");
      }
    }
  });

  it("ne garde pas de texte pour un provider sorti du catalogue", () => {
    const known = new Set(providerIds);
    for (const [lang, locale] of Object.entries(locales)) {
      const orphans = Object.keys(locale.apiKeys.providers).filter((id) => !known.has(id));
      expect(orphans, `${lang} garde des providers inconnus`).toEqual([]);
    }
  });
});
