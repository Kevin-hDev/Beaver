import i18n from "@/i18n";
import { parseForecastDoc } from "./forecast-docs-parser";
import type { ForecastDocPage } from "./forecast-docs-types";

const FORECAST_DOC_IDS = [
  "overview",
  "datasets",
  "models",
  "predictions",
  "uncertainty",
  "covariates",
  "multiseries",
  "scenarios",
  "agent-workflows",
  "tool-contracts",
  "troubleshooting",
  "limits",
] as const;

const FORECAST_DOC_NAV_KEYS: Record<(typeof FORECAST_DOC_IDS)[number], string> = {
  overview: "forecast.docs.nav.overview",
  datasets: "forecast.docs.nav.datasets",
  models: "forecast.docs.nav.models",
  predictions: "forecast.docs.nav.predictions",
  uncertainty: "forecast.docs.nav.uncertainty",
  covariates: "forecast.docs.nav.covariates",
  multiseries: "forecast.docs.nav.multiseries",
  scenarios: "forecast.docs.nav.scenarios",
  "agent-workflows": "forecast.docs.nav.agentWorkflows",
  "tool-contracts": "forecast.docs.nav.toolContracts",
  troubleshooting: "forecast.docs.nav.troubleshooting",
  limits: "forecast.docs.nav.limits",
};

// Map app locales to forecast-docs content folders. Falls back to French
// (the original source language) then English for anything missing.
const FORECAST_DOC_LOCALES = new Map<string, string>([
  ["fr", "fr"],
  ["en", "en"],
  ["es", "es"],
  ["de", "de"],
  ["it", "it"],
  ["zh", "zh"],
  ["ja", "ja"],
]);

// Per-locale raw markdown imports. Each locale loads every doc id; the loader
// picks the best available locale using the fallback chain below.
const FORECAST_DOC_FALLBACK_CHAIN = ["fr", "en"];

type DocBundle = Record<(typeof FORECAST_DOC_IDS)[number], string>;

async function loadLocale(locale: string): Promise<DocBundle | null> {
  const folder = FORECAST_DOC_LOCALES.get(locale);
  if (!folder) return null;
  try {
    const entries = await Promise.all(
      FORECAST_DOC_IDS.map(async (id): Promise<[string, string]> => {
        const mod = (await import(`@/content/forecast-docs/${folder}/${id}.md?raw`)) as {
          default: string;
        };
        return [id, mod.default];
      }),
    );
    return Object.fromEntries(entries) as DocBundle;
  } catch {
    return null;
  }
}

export async function loadForecastDocPages(): Promise<ForecastDocPage[]> {
  const current = i18n.language?.split("-")[0] ?? "en";
  const wanted = FORECAST_DOC_LOCALES.has(current) ? current : "en";

  const chain = [wanted, ...FORECAST_DOC_FALLBACK_CHAIN.filter((l) => l !== wanted)];
  const bundles: DocBundle[] = [];
  for (const locale of chain) {
    const bundle = await loadLocale(locale);
    if (bundle) bundles.push(bundle);
    if (bundles.length >= 2) break;
  }

  if (bundles.length === 0) return [];

  const primary = bundles[0];
  const fallback = bundles[1];

  return FORECAST_DOC_IDS.map((id) => {
    const markdown = primary[id] ?? fallback?.[id] ?? "";
    const navLabel = FORECAST_DOC_NAV_KEYS[id];
    return parseForecastDoc({ id, navLabel, markdown });
  });
}
