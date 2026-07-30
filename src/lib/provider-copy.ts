// Textes affichés des providers. Ils vivent dans src/i18n/*.json ; les
// catalogues Rust ne gardent que le technique (URL, catégorie, plafonds).

import type { TFunction } from "i18next";
import type { ProviderCategory } from "@/types/api";

type ProviderField = "description" | "freeTier";

/** Un `ProviderSpec` suffit ; les écrans de prévision passent un objet minimal. */
export interface ProviderRef {
  id: string;
  category?: ProviderCategory;
}

/**
 * La prévision a sa propre section : l'id `google` désigne Gemini dans le
 * catalogue LLM et TimesFM dans celui de prévision. Un espace de noms commun
 * afficherait le texte de l'un sous le nom de l'autre.
 */
function sectionFor(category: ProviderCategory | undefined): string {
  return category === "forecast" ? "forecast.providers" : "apiKeys.providers";
}

/**
 * `lng` force une langue précise — utilisé par la recherche du catalogue, qui
 * doit continuer à trouver un provider sur son libellé anglais.
 */
function providerText(
  t: TFunction,
  provider: ProviderRef,
  field: ProviderField,
  lng?: string,
): string {
  // defaultValue vide : un provider ajouté au catalogue Rust sans ses
  // traductions n'affiche pas sa clé technique à l'utilisateur. L'oubli est
  // rattrapé par api-provider-translations.test.ts, pas par l'interface.
  const key = `${sectionFor(provider.category)}.${provider.id}.${field}`;
  return t(key, { defaultValue: "", lng });
}

export function providerDescription(
  t: TFunction,
  provider: ProviderRef,
  lng?: string,
): string {
  return providerText(t, provider, "description", lng);
}

export function providerFreeTier(t: TFunction, provider: ProviderRef): string {
  return providerText(t, provider, "freeTier");
}
