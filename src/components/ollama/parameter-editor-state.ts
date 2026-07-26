import {
  MODEL_PARAMETER_DEFINITIONS,
  isOfficialParameterKey,
  type SingleValueParameterKey,
} from "./model-parameter-catalog";
import type { ModelParameter } from "./modelfile-utils";

export const MAX_PARAMETER_ENTRIES = 128;
export const MAX_STOP_SEQUENCES = 32;
export const MAX_CUSTOM_PARAMETERS = 64;
export const MAX_PARAMETER_KEY_LENGTH = 64;
export const MAX_PARAMETER_VALUE_LENGTH = 1024;

const CUSTOM_PARAMETER_KEY = /^[a-zA-Z][a-zA-Z0-9_]*$/;

export interface ParameterEditorState {
  values: Record<SingleValueParameterKey, string>;
  stopValues: string[];
  stopIds: string[];
  customParameters: ModelParameter[];
  customParameterIds: string[];
}

export function createParameterEditorState(
  initialParameters: ModelParameter[],
): ParameterEditorState {
  const values = emptyOfficialValues();
  const stopValues: string[] = [];
  const customParameters: ModelParameter[] = [];

  for (const parameter of initialParameters.slice(0, MAX_PARAMETER_ENTRIES)) {
    const normalizedKey = parameter.key.trim().toLowerCase();
    if (normalizedKey === "stop") {
      if (stopValues.length < MAX_STOP_SEQUENCES) stopValues.push(parameter.value);
      continue;
    }
    if (isOfficialParameterKey(normalizedKey)) {
      values[normalizedKey as SingleValueParameterKey] = parameter.value;
      continue;
    }
    if (customParameters.length < MAX_CUSTOM_PARAMETERS) {
      customParameters.push({ key: parameter.key, value: parameter.value });
    }
  }

  return {
    values,
    stopValues: stopValues.length > 0 ? stopValues : [""],
    stopIds: Array.from(
      { length: Math.max(stopValues.length, 1) },
      createParameterRowId,
    ),
    customParameters,
    customParameterIds: Array.from({ length: customParameters.length }, createParameterRowId),
  };
}

export function buildParameterPayload(state: ParameterEditorState): Array<[string, string]> {
  const payload: Array<[string, string]> = [];
  for (const definition of MODEL_PARAMETER_DEFINITIONS) {
    if (definition.key === "stop") {
      for (const value of state.stopValues) pushEntry(payload, "stop", value);
      continue;
    }
    pushEntry(payload, definition.key, state.values[definition.key]);
  }
  for (const parameter of state.customParameters) {
    pushEntry(payload, parameter.key, parameter.value);
  }
  return payload;
}

export function hasInvalidCustomParameter(state: ParameterEditorState): boolean {
  return state.customParameters.some(({ key, value }) => {
    const trimmedKey = key.trim();
    const trimmedValue = value.trim();
    if (!trimmedKey && !trimmedValue) return false;
    return !CUSTOM_PARAMETER_KEY.test(trimmedKey)
      || isOfficialParameterKey(trimmedKey.toLowerCase())
      || trimmedKey.length > MAX_PARAMETER_KEY_LENGTH
      || trimmedValue.length > MAX_PARAMETER_VALUE_LENGTH;
  });
}

export function hasInvalidOfficialParameter(state: ParameterEditorState): boolean {
  return MODEL_PARAMETER_DEFINITIONS.some((definition) => {
    if (definition.key === "stop" || definition.valueType === "text") return false;
    const value = state.values[definition.key].trim();
    if (!value) return false;
    if (definition.valueType === "integer") return !/^[+-]?\d+$/.test(value);
    return !Number.isFinite(Number(value));
  });
}

export function createParameterRowId(): string {
  return globalThis.crypto.randomUUID();
}

function emptyOfficialValues(): Record<SingleValueParameterKey, string> {
  return Object.fromEntries(
    MODEL_PARAMETER_DEFINITIONS
      .filter((definition) => definition.key !== "stop")
      .map((definition) => [definition.key, ""]),
  ) as Record<SingleValueParameterKey, string>;
}

function pushEntry(payload: Array<[string, string]>, key: string, value: string): void {
  const trimmedKey = key.trim();
  const trimmedValue = value.trim();
  if (trimmedKey && trimmedValue) payload.push([trimmedKey, trimmedValue]);
}
