import { LIMITS } from "@/types/extension-contract.generated";
import {
  UI_LIMITS,
  UI_LOCALES,
} from "@/types/extension-ui-contract.generated";
import { isExtensionIdentifier } from "@/lib/extension-records";
import type { StandardFieldValue, StandardLocalizedText } from "./types";

export function plain(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw invalid();
  return value as Record<string, unknown>;
}

export function exact(value: Record<string, unknown>, allowed: readonly string[]): void {
  const keys = Object.keys(value);
  if (keys.length > allowed.length || keys.some((key) => !allowed.includes(key))) throw invalid();
}

export function identifier(value: unknown): string {
  if (typeof value !== "string" || value.length > LIMITS.maxIdentifierChars
    || !isExtensionIdentifier(value)) throw invalid();
  return value;
}

export function ownedIdentifier(extensionId: string, value: unknown): string {
  const parsed = identifier(value);
  if (!parsed.startsWith(`${extensionId}.`)) throw invalid();
  return parsed;
}

export function localized(value: unknown): StandardLocalizedText {
  const input = plain(value);
  exact(input, ["default", ...UI_LOCALES]);
  if (!("default" in input)) throw invalid();
  for (const item of Object.values(input)) {
    if (typeof item !== "string" || item.length === 0
      || Array.from(item).length > UI_LIMITS.maxTextChars) throw invalid();
  }
  return input as StandardLocalizedText;
}

export function fieldValue(value: unknown): StandardFieldValue {
  if (value === null || typeof value === "boolean"
    || (typeof value === "number" && Number.isFinite(value))
    || (typeof value === "string" && Array.from(value).length <= UI_LIMITS.maxTextChars)) {
    return value;
  }
  throw invalid();
}

export function jsonBytes(value: unknown): number {
  try {
    return new TextEncoder().encode(JSON.stringify(value)).byteLength;
  } catch {
    throw invalid();
  }
}

export function invalid(): Error {
  return new Error("invalid_extension_ui_catalog");
}
