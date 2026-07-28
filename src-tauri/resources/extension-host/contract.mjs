import { readFileSync } from "node:fs";

const MAX_CONTRACT_BYTES = 8_192;
const CODE_PATTERN = /^[a-z][a-z0-9_]{0,63}$/;
const raw = readFileSync(new URL("./contract.json", import.meta.url));
if (raw.length > MAX_CONTRACT_BYTES) throw new Error("invalid_extension_contract");

const contract = JSON.parse(raw.toString("utf8"));
const limit = (name) => {
  const value = contract?.limits?.[name];
  if (!Number.isSafeInteger(value) || value < 1 || value > 16_777_216) {
    throw new Error("invalid_extension_contract");
  }
  return value;
};

function codes(name) {
  const values = contract?.diagnostics?.[name];
  if (!Array.isArray(values) || values.length < 1 || values.length > 32) {
    throw new Error("invalid_extension_contract");
  }
  const unique = new Set(values);
  if (
    unique.size !== values.length
    || values.some((value) => typeof value !== "string" || !CODE_PATTERN.test(value))
  ) {
    throw new Error("invalid_extension_contract");
  }
  return Object.freeze([...values]);
}

export const LIMITS = Object.freeze({
  maxExtensions: limit("maxExtensions"),
  maxUserExtensions: limit("maxUserExtensions"),
  maxTools: limit("maxTools"),
  maxToolsPerExtension: limit("maxToolsPerExtension"),
  maxEventsPerExtension: limit("maxEventsPerExtension"),
  maxPendingRequests: limit("maxPendingRequests"),
  maxInFlightRequests: limit("maxInFlightRequests"),
  maxInFlightHandlers: limit("maxInFlightHandlers"),
  maxWorkingDirectoryChars: limit("maxWorkingDirectoryChars"),
  maxMessageBytes: limit("maxMessageBytes"),
});

export const HOST_DIAGNOSTIC_CODES = codes("hostCodes");
export const RUNTIME_DIAGNOSTIC_CODES = codes("runtimeCodes");
const allCodes = [...HOST_DIAGNOSTIC_CODES, ...RUNTIME_DIAGNOSTIC_CODES];
if (new Set(allCodes).size !== allCodes.length) {
  throw new Error("invalid_extension_contract");
}
export const DIAGNOSTIC = Object.freeze(
  Object.fromEntries(allCodes.map((code) => [code, code])),
);
