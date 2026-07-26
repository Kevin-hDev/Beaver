import type { ForecastModelReadiness } from "./forecast-model-readiness";

export {
  getForecastModelReadiness,
  isForecastModelSelectable,
  isForecastModelVisibleInSelector,
} from "./forecast-model-readiness";
export type { ForecastModelReadiness } from "./forecast-model-readiness";

export interface ForecastProviderEntry {
  id: string;
  display_name: string;
  configured?: boolean;
}

export interface ForecastModelEntry {
  id: string;
  provider_id: string;
  family_id?: string;
  display_name: string;
  params: string;
  size_mb: number;
  ram_mb: number;
  vram_mb: number | null;
  cpu_supported: boolean;
  gpu_supported: boolean;
  multivariate: boolean;
  covariates: boolean;
  horizon_max: number;
  frequencies: string;
  is_cloud: boolean;
  installed: boolean;
  runtime_ready: boolean;
  readiness_state?: ForecastModelReadiness;
  installable?: boolean;
  runnable?: boolean;
  size_on_disk?: number;
  provider_configured?: boolean;
  engine_kind?: string;
  config_params?: string[];
  interval_support?: "continuous" | "central_60_or_80";
  interval_capability?: {
    mode: "continuous" | "fixed_grid";
    supported_confidence_levels: number[];
    confidence_step: number | null;
  };
  capabilities?: {
    past_covariates: boolean;
    future_covariates: boolean;
    multi_series: boolean;
    multivariate: boolean;
    probabilistic: boolean;
    backtesting_ready: boolean;
    anomalies_ready: boolean;
    fine_tuning_ready: boolean;
  };
}

export interface ForecastCapabilitySet {
  context: boolean;
  futureContext: boolean;
  multiSeries: boolean;
  multivariate: boolean;
  probabilistic: boolean;
  backtesting: boolean;
}

export interface ForecastModelsResponse {
  providers: ForecastProviderEntry[];
  models: ForecastModelEntry[];
  configured_provider_ids: string[];
}

export interface ForecastModelGroup {
  id: string;
  titleKey: string;
  models: ForecastModelEntry[];
}

const FAMILY_ORDER = [
  "chronos-bolt",
  "chronos-2",
  "timesfm-2-5",
  "timegpt-2",
  "toto-2",
  "moirai-2",
  "flowstate",
  "tabpfn-ts",
  "tirex",
  "kairos",
  "sundial",
];

export function getForecastFamilyId(model: ForecastModelEntry): string {
  if (model.family_id) return model.family_id;
  if (model.id.startsWith("chronos-bolt")) return "chronos-bolt";
  if (model.id.startsWith("chronos-2")) return "chronos-2";
  if (model.id.startsWith("timegpt")) return "timegpt-2";
  return model.provider_id;
}

export function getForecastFamilyKey(familyId: string): string {
  return `forecast.models.families.${familyId}`;
}

export function groupForecastModels(models: ForecastModelEntry[]): ForecastModelGroup[] {
  const grouped = new Map<string, ForecastModelEntry[]>();
  models.forEach((model) => {
    const familyId = getForecastFamilyId(model);
    grouped.set(familyId, [...(grouped.get(familyId) ?? []), model]);
  });
  return [...grouped.entries()]
    .sort((a, b) => sortFamily(a[0]) - sortFamily(b[0]))
    .map(([id, familyModels]) => ({
      id,
      titleKey: getForecastFamilyKey(id),
      models: familyModels.sort(sortModels),
    }));
}

export function listForecastFamilies(models: ForecastModelEntry[]): ForecastModelGroup[] {
  const grouped = new Map(groupForecastModels(models).map((group) => [group.id, group.models]));
  const ordered = FAMILY_ORDER.map((id) => ({
    id,
    titleKey: getForecastFamilyKey(id),
    models: grouped.get(id) ?? [],
  }));
  const extras = [...grouped.entries()]
    .filter(([id]) => !FAMILY_ORDER.includes(id))
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([id, familyModels]) => ({
      id,
      titleKey: getForecastFamilyKey(id),
      models: familyModels,
    }));
  return [...ordered, ...extras];
}

export function isForecastModelConfigurable(model: ForecastModelEntry): boolean {
  if (model.is_cloud) return Boolean(model.provider_configured && model.runnable);
  return model.installed && model.installable !== false;
}

export function getForecastModelSummaryKey(modelId: string): string {
  return `forecast.models.descriptions.${modelId}`;
}

export function getForecastHardwareKey(model: ForecastModelEntry): string {
  if (model.is_cloud) return "forecast.models.hardware.remote";
  if (model.size_mb <= 64) return "forecast.models.hardware.light";
  if (model.size_mb <= 512) return "forecast.models.hardware.balanced";
  return "forecast.models.hardware.heavy";
}

export function getModelCapabilities(model: ForecastModelEntry): ForecastCapabilitySet {
  const caps = model.capabilities;
  return {
    context: Boolean(caps?.past_covariates),
    futureContext: Boolean(caps?.future_covariates),
    multiSeries: Boolean(caps?.multi_series),
    multivariate: Boolean(caps?.multivariate),
    probabilistic: Boolean(caps?.probabilistic),
    backtesting: Boolean(caps?.backtesting_ready),
  };
}

export function getForecastEngineKey(model: ForecastModelEntry): string {
  if (model.engine_kind === "local_chronos2") return "forecast.models.engines.localChronos2";
  if (model.engine_kind === "local_chronos_bolt") return "forecast.models.engines.localChronosBolt";
  if (!model.is_cloud) return "forecast.models.engines.localPackage";
  return "forecast.models.engines.cloudApi";
}

function sortFamily(familyId: string): number {
  const index = FAMILY_ORDER.indexOf(familyId);
  return index === -1 ? FAMILY_ORDER.length : index;
}

function sortModels(a: ForecastModelEntry, b: ForecastModelEntry): number {
  if (a.is_cloud !== b.is_cloud) return a.is_cloud ? 1 : -1;
  if (a.is_cloud) return a.display_name.localeCompare(b.display_name);
  return a.size_mb - b.size_mb;
}
