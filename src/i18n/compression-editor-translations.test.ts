import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

type JsonObject = Record<string, unknown>;

function compressionKeys(object: JsonObject, prefix = ""): string[] {
  return Object.entries(object).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object") {
      return compressionKeys(value as JsonObject, path);
    }
    return path.startsWith("compression") ? [path] : [];
  });
}

describe("compression editor translations", () => {
  it("garde les mêmes clés dans les sept langues", () => {
    const advanced = (locale: unknown): JsonObject => (
      locale as { settings: { advanced: JsonObject } }
    ).settings.advanced;
    const locales: unknown[] = [en, es, de, italian, zh, ja];
    const expected = compressionKeys(advanced(fr)).sort();
    for (const locale of locales) {
      expect(compressionKeys(advanced(locale)).sort()).toEqual(expected);
    }
  });

  it("n'affiche aucune interpolation orpheline sur la carte", () => {
    const locales = [fr, en, es, de, italian, zh, ja];
    for (const locale of locales) {
      expect(locale.settings.advanced.compressionAdvancedDesc).not.toContain("{{name}}");
    }
  });

  it("reprend exactement l'explication française du garde sous 64K", () => {
    expect(fr.agentLocal.contextUsage.compressionHelp).toBe(
      "La compression est désactivée parce que cette fenêtre de contexte est inférieure à 64K. Ces petites fenêtres peuvent ne pas laisser assez de place pour générer puis réinjecter un résumé, les instructions, les outils et les échanges récents. Vous pouvez l’autoriser dans le profil de compression avancé.",
    );
  });
});
