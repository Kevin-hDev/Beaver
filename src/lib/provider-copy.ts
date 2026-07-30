// Textes affichés des providers d'API. Ils vivent dans src/i18n/*.json ; le
// catalogue Rust ne garde que le technique (URL, catégorie, plafonds).

import type { TFunction } from "i18next";

const PROVIDER_SECTION = "apiKeys.providers";

type ProviderField = "description" | "freeTier";

/**
 * `lng` force une langue précise — utilisé par la recherche du catalogue, qui
 * doit continuer à trouver un provider sur son libellé anglais.
 */
function providerText(
  t: TFunction,
  providerId: string,
  field: ProviderField,
  lng?: string,
): string {
  // defaultValue vide : un provider ajouté au catalogue Rust sans ses
  // traductions n'affiche pas sa clé technique à l'utilisateur. L'oubli est
  // rattrapé par api-provider-translations.test.ts, pas par l'interface.
  return t(`${PROVIDER_SECTION}.${providerId}.${field}`, { defaultValue: "", lng });
}

export function providerDescription(
  t: TFunction,
  providerId: string,
  lng?: string,
): string {
  return providerText(t, providerId, "description", lng);
}

export function providerFreeTier(t: TFunction, providerId: string): string {
  return providerText(t, providerId, "freeTier");
}
