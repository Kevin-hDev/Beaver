export type ModelParameterGroup =
  | "context"
  | "length"
  | "creativity"
  | "repetition"
  | "sampling"
  | "stop";

export type ModelParameterValueType = "integer" | "decimal" | "text";

export type OfficialModelParameterKey =
  | "num_ctx"
  | "repeat_last_n"
  | "repeat_penalty"
  | "temperature"
  | "seed"
  | "stop"
  | "num_predict"
  | "draft_num_predict"
  | "top_k"
  | "top_p"
  | "min_p";

export type SingleValueParameterKey = Exclude<OfficialModelParameterKey, "stop">;

export interface ModelParameterDefinition {
  key: OfficialModelParameterKey;
  group: ModelParameterGroup;
  valueType: ModelParameterValueType;
  defaultValue: string | null;
}

export const MODEL_PARAMETER_GROUPS: ModelParameterGroup[] = [
  "context",
  "length",
  "creativity",
  "repetition",
  "sampling",
  "stop",
];

export const MODEL_PARAMETER_DEFINITIONS: ModelParameterDefinition[] = [
  { key: "num_ctx", group: "context", valueType: "integer", defaultValue: "auto" },
  { key: "num_predict", group: "length", valueType: "integer", defaultValue: "-1" },
  { key: "draft_num_predict", group: "length", valueType: "integer", defaultValue: "4" },
  { key: "temperature", group: "creativity", valueType: "decimal", defaultValue: "0.8" },
  { key: "seed", group: "creativity", valueType: "integer", defaultValue: "0" },
  { key: "repeat_last_n", group: "repetition", valueType: "integer", defaultValue: "64" },
  { key: "repeat_penalty", group: "repetition", valueType: "decimal", defaultValue: "1.1" },
  { key: "top_k", group: "sampling", valueType: "integer", defaultValue: "40" },
  { key: "top_p", group: "sampling", valueType: "decimal", defaultValue: "0.9" },
  { key: "min_p", group: "sampling", valueType: "decimal", defaultValue: "0.0" },
  { key: "stop", group: "stop", valueType: "text", defaultValue: null },
];

const OFFICIAL_PARAMETER_KEYS = new Set<OfficialModelParameterKey>(
  MODEL_PARAMETER_DEFINITIONS.map((definition) => definition.key),
);

export function isOfficialParameterKey(key: string): key is OfficialModelParameterKey {
  return OFFICIAL_PARAMETER_KEYS.has(key as OfficialModelParameterKey);
}
