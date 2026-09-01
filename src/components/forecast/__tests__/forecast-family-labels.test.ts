import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { getForecastFamilyLabel } from "../forecast-model-meta";

/* Chemins littéraux : la règle de sécurité du projet refuse une lecture de
   fichier dont le chemin est assemblé à l'exécution. */
const BUNDLES = {
  fr: readFileSync("src/i18n/fr.json", "utf8"),
  en: readFileSync("src/i18n/en.json", "utf8"),
  es: readFileSync("src/i18n/es.json", "utf8"),
  de: readFileSync("src/i18n/de.json", "utf8"),
  it: readFileSync("src/i18n/it.json", "utf8"),
  zh: readFileSync("src/i18n/zh.json", "utf8"),
  ja: readFileSync("src/i18n/ja.json", "utf8"),
};

const LOCALES = Object.keys(BUNDLES) as (keyof typeof BUNDLES)[];

/* Les familles viennent du registre Rust, seule autorité : une famille ajoutée
   là-bas sans son nom ici s'afficherait à l'écran sous son identifiant. */
function familiesFromRegistry(): string[] {
  const source = readFileSync("src-tauri/src/services/forecast/registry_specs.rs", "utf8");
  const found = new Set<string>();
  for (const match of source.matchAll(/rt\(\s*"[^"]+"\s*,\s*"([^"]+)"/g)) {
    found.add(match[1]);
  }
  return [...found];
}

function translations(locale: keyof typeof BUNDLES): Record<string, string> {
  const bundle = JSON.parse(BUNDLES[locale]) as {
    forecast: { models: { families?: Record<string, string> } };
  };
  return bundle.forecast.models.families ?? {};
}

describe("Noms des familles de modèles forecast", () => {
  it("nomme dans les sept langues chaque famille du registre", () => {
    const families = familiesFromRegistry();
    expect(families.length).toBeGreaterThan(0);
    for (const locale of LOCALES) {
      const named = translations(locale);
      for (const family of families) {
        expect(named[family], `${family} manque en ${locale}`).toBeTruthy();
      }
    }
  });

  it("donne le même nom propre dans les sept langues", () => {
    // Chronos-Bolt, TiRex, Toto-2 : des noms de produits, pas des mots.
    const reference = translations("fr");
    for (const locale of LOCALES) {
      expect(translations(locale)).toEqual(reference);
    }
  });

  it("retombe sur l'identifiant plutôt que d'afficher la clé", () => {
    const passthrough = (key: string) => key;
    expect(getForecastFamilyLabel("famille-inconnue", passthrough)).toBe("famille-inconnue");
  });

  it("rend le nom traduit quand il existe", () => {
    const resolve = (key: string) =>
      key === "forecast.models.families.tirex" ? "TiRex" : key;
    expect(getForecastFamilyLabel("tirex", resolve)).toBe("TiRex");
  });
});
