import { LIMITS } from "./contract.mjs";

export function validContribution(value, validIdentifier, keys) {
  return value
    && typeof value === "object"
    && Object.keys(value).every((key) => keys.includes(key))
    && validIdentifier(value.id)
    && validText(value.name, LIMITS.maxExtensionNameChars)
    && validText(value.description, LIMITS.maxExtensionTextChars)
    && typeof value.path === "string"
    && Array.from(value.path).length <= LIMITS.maxPathChars;
}

export function validRelativePath(value) {
  return typeof value === "string"
    && value.length > 0
    && !value.startsWith("/")
    && !value.includes("\\")
    && !value.includes(":")
    && !/[\0-\x1F\x7F-\x9F]/u.test(value)
    && value.split("/").every((part) => part && part !== "." && part !== ".." && !dosReserved(part));
}

function dosReserved(part) {
  const base = part.replace(/[ .]+$/u, "").split(".")[0].replace(/ +$/u, "").toUpperCase();
  return /^(?:CON|PRN|AUX|NUL|(?:COM|LPT)[1-9¹²³])$/u.test(base);
}

export function unicodeScalarLength(value) {
  return Array.from(value).length;
}

function validText(value, maximum) {
  return typeof value === "string" && value.trim() && Array.from(value).length <= maximum;
}
