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
    expect(providerDescription(i18n.t, "groq")).toBe(
      "Inférence ultra-rapide Llama / GPT-OSS sur LPU custom.",
    );
    expect(providerFreeTier(i18n.t, "firecrawl")).toBe("500 crédits");
  });

  it("suit le changement de langue", async () => {
    await i18n.changeLanguage("ja");
    expect(providerDescription(i18n.t, "moonshot")).toContain("Kimi K3");
    expect(providerFreeTier(i18n.t, "firecrawl")).toBe("500 クレジット");
    await i18n.changeLanguage("fr");
  });

  it("force l'anglais quand la recherche le demande", () => {
    expect(providerDescription(i18n.t, "exa", "en")).toBe(
      "Neural search — semantic similarity search.",
    );
  });

  it("rend une chaîne vide pour un provider sans traduction", () => {
    // Jamais la clé technique : elle exposerait la structure interne à l'écran.
    expect(providerDescription(i18n.t, "provider-fantome")).toBe("");
    expect(providerFreeTier(i18n.t, "provider-fantome")).toBe("");
  });
});
