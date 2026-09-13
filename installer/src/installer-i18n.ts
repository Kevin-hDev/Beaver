import de from "../../src/i18n/de.json";
import en from "../../src/i18n/en.json";
import es from "../../src/i18n/es.json";
import fr from "../../src/i18n/fr.json";
import it from "../../src/i18n/it.json";
import ja from "../../src/i18n/ja.json";
import zh from "../../src/i18n/zh.json";

const resources = { de, en, es, fr, it, ja, zh } as const;
type Locale = keyof typeof resources;

declare global {
  interface Window {
    __BEAVER_INSTALLER_LOCALE__?: string;
  }
}

function normalizeLocale(locale: string | undefined): Locale {
  const base = locale?.toLowerCase().split("-")[0];
  return base && base in resources ? (base as Locale) : "en";
}

export function installerLocale(): Locale {
  return normalizeLocale(window.__BEAVER_INSTALLER_LOCALE__ ?? document.documentElement.lang);
}

export function t(key: string, variables: Record<string, string | number> = {}): string {
  const value = key.split(".").reduce<unknown>((current, part) => {
    if (!current || typeof current !== "object" || !(part in current)) return undefined;
    return (current as Record<string, unknown>)[part];
  }, resources[installerLocale()]);
  const fallback = key.split(".").reduce<unknown>((current, part) => {
    if (!current || typeof current !== "object" || !(part in current)) return undefined;
    return (current as Record<string, unknown>)[part];
  }, resources.en);
  const text = typeof value === "string" ? value : typeof fallback === "string" ? fallback : key;
  return Object.entries(variables).reduce(
    (result, [name, replacement]) => result.replaceAll(`{{${name}}}`, String(replacement)),
    text,
  );
}
