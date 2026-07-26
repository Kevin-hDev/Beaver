// Types frontend alignés sur le backend Rust (src-tauri/src/services/api_keys.rs).
// Les provider_ids doivent rester cohérents entre Rust et TS.

export type ProviderCategory = "llm" | "search" | "scraping" | "forecast";

/**
 * Spec d'un provider (miroir du catalog Rust).
 */
export interface ProviderSpec {
  id: string;
  display_name: string;
  category: ProviderCategory;
  signup_url: string;
  free_tier_label: string;
  short_description: string;
  short_description_en: string;
  /** Pour les providers LLM — absent pour search/scraping */
  base_url?: string;
  models_endpoint?: string;
}

export function getProviderDescription(provider: ProviderSpec, lang: string): string {
  return lang === "fr" ? provider.short_description : provider.short_description_en;
}
