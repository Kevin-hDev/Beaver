import type { AvailableModel } from "@/hooks/use-available-models";
import {
  getForecastFamilyId,
  getForecastFamilyKey,
  groupForecastModels,
  getForecastModelReadiness,
  isForecastModelSelectable,
  type ForecastModelEntry,
} from "../forecast-model-meta";

type Translate = (key: string) => string;

export function buildForecastSelectorGroups(
  models: ForecastModelEntry[],
  query: string,
  t: Translate,
): Map<string, AvailableModel[]> {
  const lowered = query.trim().toLowerCase();
  const visible = lowered
    ? models.filter((model) =>
        `${model.id} ${model.display_name}`.toLowerCase().includes(lowered))
    : models;
  const mapped = new Map<string, AvailableModel[]>();
  for (const group of groupForecastModels(visible)) {
    const familyName = t(group.titleKey);
    mapped.set(group.id, group.models.map((model) => {
      const disabled = !isForecastModelSelectable(model);
      const updateRequired = getForecastModelReadiness(model) === "update_required";
      return {
        id: model.id,
        display_name: model.display_name,
        provider_id: getForecastFamilyId(model),
        provider_name: familyName === getForecastFamilyKey(group.id) ? group.id : familyName,
        is_local: !model.is_cloud,
        supports_tools: false,
        supports_vision: false,
        is_free: true,
        hint: disabled ? undefined : model.params,
        disabled,
        disabled_hint: disabled
          ? t(updateRequired
              ? "forecast.models.updateRequired"
              : "forecast.models.preparationRequired")
          : undefined,
      };
    }));
  }
  return mapped;
}

export function firstSelectableModel(
  models: ForecastModelEntry[],
): ForecastModelEntry | undefined {
  return models.find(isForecastModelSelectable);
}
