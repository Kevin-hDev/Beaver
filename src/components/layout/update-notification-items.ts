import type { AppUpdate, OllamaModelUpdate, OllamaBinaryUpdate } from "@/hooks/use-update-checker";
import type { ForecastDevUpdate } from "@/hooks/use-forecast-dev-updates";
import { BRAND } from "@/lib/brand";
import type { ItemData } from "./bubble-item";

export function buildUpdateItems(
  t: (k: string, opts?: Record<string, string>) => string,
  language: string,
  app: AppUpdate | null,
  binary: OllamaBinaryUpdate | null,
  models: OllamaModelUpdate[],
  forecastUpdates: ForecastDevUpdate[],
): ItemData[] {
  const items: ItemData[] = [];
  if (app) {
    items.push({
      id: "app",
      type: "app",
      name: BRAND.displayName,
      sub: t("updates.version", { version: app.version }),
      version: app.version,
      title: app.title,
      publishedAt: app.publishedAt,
      notesByLocale: app.notesByLocale,
      language,
      assetUrl: app.assetUrl,
      dismissUpdate: { kind: "app", subject: "beaver", version: app.version },
    });
  }
  if (binary) {
    items.push({
      id: "ollama-binary",
      type: "ollama-binary",
      name: "Ollama",
      sub: `v${binary.currentVersion} → v${binary.latestVersion}`,
      dismissUpdate: { kind: "ollama_binary", subject: "ollama", version: binary.latestVersion },
    });
  }
  for (const m of models) {
    items.push({
      id: m.fullName,
      type: "ollama",
      name: m.fullName,
      sub: m.family,
      fullName: m.fullName,
      dismissUpdate: { kind: "ollama_model", subject: m.fullName, version: m.latestDigest },
    });
  }
  for (const update of forecastUpdates) {
    const current = shortVersion(update.current);
    const latest = shortVersion(update.latest);
    items.push({
      id: `forecast-dev-${update.id}`,
      type: "forecast-dev",
      name: update.displayName,
      sub: `${t(`updates.forecastDev${update.kind === "model" ? "Model" : "Runtime"}`)} · ${current} → ${latest}`,
      sourceUrl: update.sourceUrl,
    });
  }
  return items;
}

function shortVersion(value: string): string {
  return /^[a-f\d]{40}$/i.test(value) ? value.slice(0, 7) : value;
}
