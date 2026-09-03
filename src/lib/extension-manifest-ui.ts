import type { ExtensionManifest } from "@/types/extensions";
import {
  EXTENSION_UI_API_VERSION,
  UI_MODES,
} from "@/types/extension-ui-contract.generated";

export function parseExtensionManifestUi(
  value: unknown,
  maxEntryChars: number,
): ExtensionManifest["ui"] {
  if (value === null || value === undefined) return undefined;
  if (typeof value !== "object" || Array.isArray(value)) throw invalid();
  const input = value as Record<string, unknown>;
  if (Object.keys(input).some((key) => !["apiVersion", "mode", "entry"].includes(key))) {
    throw invalid();
  }
  const mode = isUiMode(input.mode) ? input.mode : null;
  const entry = input.entry === null || input.entry === undefined
    ? undefined
    : boundedText(input.entry, maxEntryChars);
  if (input.apiVersion !== EXTENSION_UI_API_VERSION || mode === null
    || (mode === "standard" && entry !== undefined)
    || (mode === "advanced" && entry === undefined)) {
    throw invalid();
  }
  return { apiVersion: EXTENSION_UI_API_VERSION, mode, ...(entry ? { entry } : {}) };
}

function isUiMode(value: unknown): value is (typeof UI_MODES)[number] {
  return typeof value === "string" && UI_MODES.some((mode) => mode === value);
}

function boundedText(value: unknown, maximum: number): string {
  if (typeof value !== "string" || value.length > maximum * 2
    || value.length === 0 || Array.from(value).length > maximum) {
    throw invalid();
  }
  return value;
}

function invalid(): Error {
  return new Error("invalid_extension_response");
}
