import type { ForecastConfigParam } from "../forecast/model-browser/forecast-config-types";
import type { SelectOption } from "./settings-select";

export function valueToText(value: unknown): string {
  if (Array.isArray(value)) return value.join(", ");
  if (typeof value === "boolean") return value ? "true" : "false";
  if (typeof value === "number" || typeof value === "string") return String(value);
  return "";
}

export function toDraft(params: ForecastConfigParam[]): Record<string, string> {
  return Object.fromEntries(params.map((param) => [param.id, valueToText(param.value)]));
}

export function buildPayload(params: ForecastConfigParam[], draft: Record<string, string>) {
  return Object.fromEntries(
    params.map((param) => [param.id, toPayloadValue(param, draft[param.id])]),
  );
}

function toPayloadValue(param: ForecastConfigParam, value: string | undefined) {
  if (!value) return null;
  if (param.kind === "boolean") return value === "true";
  return value;
}

export function selectOptions(
  param: ForecastConfigParam,
  t: (key: string) => string,
): SelectOption[] {
  if (param.kind === "boolean") {
    return [
      { value: "true", label: t("forecast.modelConfig.enabled") },
      { value: "false", label: t("forecast.modelConfig.disabled") },
    ];
  }
  return param.options.map((option) => ({ value: option, label: option }));
}
