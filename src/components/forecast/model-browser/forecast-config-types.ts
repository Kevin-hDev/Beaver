type ForecastConfigKind =
  | "integer"
  | "number"
  | "boolean"
  | "select"
  | "number_list";

export interface ForecastConfigParam {
  id: string;
  kind: ForecastConfigKind;
  label_key: string;
  description_key: string;
  default_value: unknown;
  value: unknown;
  effective_value: unknown;
  min?: number | null;
  max?: number | null;
  options: string[];
}

export interface ForecastModelConfig {
  model_id: string;
  family_id: string;
  params: ForecastConfigParam[];
  inherited: ForecastConfigParam[];
}
