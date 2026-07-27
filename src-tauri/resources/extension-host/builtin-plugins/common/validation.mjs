import { OFFICE_LIMITS } from "./constants.mjs";
import { rejectOffice } from "./errors.mjs";

export function requiredString(value, maxChars = OFFICE_LIMITS.maxTextChars) {
  if (
    typeof value !== "string"
    || value.length === 0
    || value.length > maxChars
    || containsInvalidXmlCharacter(value)
  ) {
    rejectOffice("invalid_input");
  }
  return value;
}

export function optionalString(value, maxChars) {
  if (value === undefined) return undefined;
  return requiredString(value, maxChars);
}

export function boundedArray(value, maxItems, allowEmpty = false) {
  if (
    !Array.isArray(value)
    || value.length > maxItems
    || (!allowEmpty && value.length === 0)
  ) {
    rejectOffice("invalid_input");
  }
  return value;
}

export function plainObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    rejectOffice("invalid_input");
  }
  return value;
}

export function scalar(value) {
  if (
    value !== null
    && typeof value !== "string"
    && typeof value !== "number"
    && typeof value !== "boolean"
  ) {
    rejectOffice("invalid_input");
  }
  if (
    (typeof value === "string" && value.length > 32_767)
    || (typeof value === "number" && !Number.isFinite(value))
  ) {
    rejectOffice("invalid_input");
  }
  return value;
}

export function containsInvalidXmlCharacter(value) {
  for (const character of value) {
    const codePoint = character.codePointAt(0);
    if (
      codePoint === 0
      || (codePoint >= 0x01 && codePoint <= 0x08)
      || codePoint === 0x0b
      || codePoint === 0x0c
      || (codePoint >= 0x0e && codePoint <= 0x1f)
      || (codePoint >= 0xd800 && codePoint <= 0xdfff)
      || codePoint === 0xfffe
      || codePoint === 0xffff
    ) {
      return true;
    }
  }
  return false;
}
