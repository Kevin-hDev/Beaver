import { OFFICE_LIMITS } from "./constants.mjs";

export function createResultBudget() {
  let remaining = OFFICE_LIMITS.maxPreviewBytes;
  return Object.freeze({
    reserve(bytes) {
      if (!Number.isSafeInteger(bytes) || bytes < 0 || bytes > remaining) {
        return false;
      }
      remaining -= bytes;
      return true;
    },
    take(value, overhead = 1) {
      const bytes = jsonBytes(value) + overhead;
      if (bytes > remaining) return false;
      remaining -= bytes;
      return true;
    },
    takeText(value, overhead = 1) {
      const fullBytes = jsonBytes(value) + overhead;
      if (fullBytes <= remaining) {
        remaining -= fullBytes;
        return { value, truncated: false };
      }
      let minimum = 0;
      let maximum = value.length;
      let accepted = "";
      while (minimum <= maximum) {
        const middle = Math.floor((minimum + maximum) / 2);
        const candidate = safeSlice(value, middle);
        if (jsonBytes(candidate) + overhead <= remaining) {
          accepted = candidate;
          minimum = middle + 1;
        } else {
          maximum = middle - 1;
        }
      }
      remaining -= Math.min(remaining, jsonBytes(accepted) + overhead);
      return { value: accepted, truncated: true };
    },
  });
}

export function structuredResultBytes(payload) {
  return Buffer.byteLength(JSON.stringify(payload), "utf8");
}

function jsonBytes(value) {
  return Buffer.byteLength(JSON.stringify(value), "utf8");
}

function safeSlice(value, end) {
  const adjusted = end > 0
    && end < value.length
    && isHighSurrogate(value.charCodeAt(end - 1))
    && isLowSurrogate(value.charCodeAt(end))
    ? end - 1
    : end;
  return value.slice(0, adjusted);
}

function isHighSurrogate(value) {
  return value >= 0xd800 && value <= 0xdbff;
}

function isLowSurrogate(value) {
  return value >= 0xdc00 && value <= 0xdfff;
}
