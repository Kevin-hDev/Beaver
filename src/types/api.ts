// Types frontend alignés sur le backend Rust (src-tauri/src/services/api_keys.rs).
// Les provider_ids doivent rester cohérents entre Rust et TS.

export type ProviderCategory = "llm" | "search" | "scraping" | "forecast";

/**
 * Spec d'un provider (miroir du catalog Rust). Ne porte que du technique : les
 * textes affichés viennent de l'i18n via @/lib/provider-copy.
 */
export interface ProviderSpec {
  id: string;
  display_name: string;
  category: ProviderCategory;
  signup_url: string;
  /** Pour les providers LLM — absent pour search/scraping */
  base_url?: string;
  models_endpoint?: string;
}
