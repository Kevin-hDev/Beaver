import errorContract from "./ollama-runtime-error-contract.json";
import type { OllamaErrorCode } from "@/types/ollama-runtime";

const GENERIC_ERROR_KEY = "ollama.errors.generic" as const;
const MAX_ERROR_CODE_LENGTH = 96;
const ERROR_KEYS = Object.freeze(errorContract) as Readonly<Record<string, string>>;

const PROGRESS_KEYS = Object.freeze({
  preparing: "ollamaSetup.extracting",
  downloading: "ollamaSetup.downloading",
  verifying: "ollamaSetup.verifying",
  extracting: "ollamaSetup.extracting",
  validating: "ollamaSetup.verifying",
  committing: "ollamaSetup.extracting",
  starting: "ollamaSetup.starting",
  recovering: "ollama.errors.generic",
  rolling_back: "ollama.errors.generic",
  cleaning: "ollamaSetup.extracting",
} as const);

export type OllamaErrorTranslationKey = `ollama.errors.${string}`;
export type OllamaProgressTranslationKey = typeof PROGRESS_KEYS[keyof typeof PROGRESS_KEYS];

export function ollamaErrorKey(value: unknown): OllamaErrorTranslationKey {
  if (typeof value !== "string" || value.length > MAX_ERROR_CODE_LENGTH) return GENERIC_ERROR_KEY;
  return Object.prototype.hasOwnProperty.call(ERROR_KEYS, value)
    ? ERROR_KEYS[value] as OllamaErrorTranslationKey
    : GENERIC_ERROR_KEY;
}

export function isOllamaErrorCode(value: unknown): value is OllamaErrorCode {
  return typeof value === "string"
    && value.length <= MAX_ERROR_CODE_LENGTH
    && Object.prototype.hasOwnProperty.call(ERROR_KEYS, value);
}

export function ollamaProgressKey(value: unknown): OllamaProgressTranslationKey {
  if (typeof value !== "string" || value.length > MAX_ERROR_CODE_LENGTH) return GENERIC_ERROR_KEY;
  return Object.prototype.hasOwnProperty.call(PROGRESS_KEYS, value)
    ? PROGRESS_KEYS[value as keyof typeof PROGRESS_KEYS]
    : GENERIC_ERROR_KEY;
}
