import { createInstance } from "i18next";
import { beforeAll, describe, expect, it } from "vitest";

import en from "@/i18n/en.json";
import fr from "@/i18n/fr.json";
import ja from "@/i18n/ja.json";
import { providerDescription, providerFreeTier } from "./provider-copy";

// Instance dédiée : on veut vérifier la résolution réelle des clés, ce que les
// tests de composants ne peuvent pas faire puisqu'ils mockent react-i18next.
const i18n = createInstance();

beforeAll(async () => {
  await i18n.init({
    resources: {
      fr: { translation: fr },
      en: { translation: en },
      ja: { translation: ja },
    },
    lng: "fr",
    fallbackLng: "en",
    interpolation: { escapeValue: false },
  });
});

describe("provider-copy", () => {
  it("rend la description dans la langue active", () => {
    expect(providerDescription(i18n.t, { id: "groq", category: "llm" })).toBe(
      "Inférence ultra-rapide Llama / GPT-OSS sur LPU custom.",
    );
    expect(providerFreeTier(i18n.t, { id: "firecrawl", category: "scraping" })).toBe(
      "1 000 crédits/mois",
    );
  });

  it("sépare les catalogues qui partagent un id", () => {
    // `google` désigne Gemini côté clés API et TimesFM côté prévision : une
    // section commune afficherait le texte de l'un sous le nom de l'autre.
    expect(providerDescription(i18n.t, { id: "google", category: "llm" })).toContain("Gemini");
    expect(providerDescription(i18n.t, { id: "google", category: "forecast" })).toContain("TimesFM");
    expect(providerFreeTier(i18n.t, { id: "nixtla", category: "forecast" })).toBe("Aperçu / API");
  });

  it("suit le changement de langue", async () => {
    await i18n.changeLanguage("ja");
    expect(providerDescription(i18n.t, { id: "moonshot", category: "llm" })).toContain("Kimi K3");
    expect(providerFreeTier(i18n.t, { id: "firecrawl", category: "scraping" })).toBe(
      "毎月 1,000 クレジット",
    );
    expect(providerFreeTier(i18n.t, { id: "chronos", category: "forecast" })).toBe("ローカル");
    await i18n.changeLanguage("fr");
  });

  it("force l'anglais quand la recherche le demande", () => {
    expect(providerDescription(i18n.t, { id: "exa", category: "search" }, "en")).toBe(
      "Neural search — semantic similarity search.",
    );
  });

  it("rend une chaîne vide pour un provider sans traduction", () => {
    // Jamais la clé technique : elle exposerait la structure interne à l'écran.
    expect(providerDescription(i18n.t, { id: "provider-fantome" })).toBe("");
    expect(providerFreeTier(i18n.t, { id: "provider-fantome" })).toBe("");
  });
});
