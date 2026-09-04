import { LIMITS } from "@/types/extension-contract.generated";

export function invalid(): never {
  throw new Error("invalid_extension_response");
}

export function object(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) invalid();
  return value as Record<string, unknown>;
}

export function objectWithKeys(
  value: unknown,
  keys: readonly string[],
): Record<string, unknown> {
  const input = object(value);
  if (Object.keys(input).some((key) => !keys.includes(key))) invalid();
  return input;
}

export function text(value: unknown, maxChars: number, allowEmpty = false): string {
  if (typeof value !== "string") invalid();
  const length = Array.from(value).length;
  if (length > maxChars || (!allowEmpty && length === 0)) invalid();
  return value;
}

export function optionalText(value: unknown, maxChars: number): string | undefined {
  return value === null || value === undefined ? undefined : text(value, maxChars);
}

export function identifier(value: unknown): string {
  const parsed = text(value, LIMITS.maxIdentifierChars);
  if (!isExtensionIdentifier(parsed)) invalid();
  return parsed;
}

export function isExtensionIdentifier(value: string): boolean {
  const characters = Array.from(value);
  return characters.length <= LIMITS.maxIdentifierChars
    && asciiAlphanumeric(characters[0])
    && asciiAlphanumeric(characters[characters.length - 1])
    && characters.every((character) =>
      asciiAlphanumeric(character) || [".", "_", "-"].includes(character));
}

export function oneOf<T extends string>(value: unknown, values: readonly T[]): T {
  if (typeof value !== "string" || !values.includes(value as T)) invalid();
  return value as T;
}

function asciiAlphanumeric(character: string | undefined): boolean {
  if (!character) return false;
  const code = character.charCodeAt(0);
  return (code >= 48 && code <= 57)
    || (code >= 65 && code <= 90)
    || (code >= 97 && code <= 122);
}
