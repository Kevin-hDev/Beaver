import { closeSync, openSync, readSync } from "node:fs";

export const BOOTSTRAP_FILE_MAX_BYTES = 256;
export const MAX_BOOTSTRAPPED_CONTRACT_BYTES = 1_048_576;
function readBounded(url, maximum) {
  const descriptor = openSync(url, "r");
  const bytes = Buffer.allocUnsafe(maximum + 1);
  let length = 0;
  try {
    while (length <= maximum) {
      const count = readSync(descriptor, bytes, length, maximum + 1 - length, null);
      if (count === 0) break;
      length += count;
    }
  } finally {
    closeSync(descriptor);
  }
  if (length > maximum) throw new Error("invalid_extension_contract");
  return bytes.subarray(0, length);
}

function parseJson(bytes) {
  try {
    return JSON.parse(bytes.toString("utf8"));
  } catch {
    throw new Error("invalid_extension_contract");
  }
}

const bootstrap = parseJson(readBounded(
  new URL("./contract-bootstrap.json", import.meta.url),
  BOOTSTRAP_FILE_MAX_BYTES,
));
if (
  !bootstrap
  || typeof bootstrap !== "object"
  || Array.isArray(bootstrap)
  || Object.keys(bootstrap).length !== 1
  || !Number.isSafeInteger(bootstrap.maxContractBytes)
  || bootstrap.maxContractBytes < 1
  || bootstrap.maxContractBytes > MAX_BOOTSTRAPPED_CONTRACT_BYTES
) {
  throw new Error("invalid_extension_contract");
}

const contract = parseJson(readBounded(
  new URL("./contract.json", import.meta.url),
  bootstrap.maxContractBytes,
));
const maximum = contract?.validation?.maxNumericLimit;
if (!Number.isSafeInteger(maximum) || maximum < 1) {
  throw new Error("invalid_extension_contract");
}

function numericObject(name) {
  const values = contract?.[name];
  if (!values || typeof values !== "object" || Array.isArray(values)) {
    throw new Error("invalid_extension_contract");
  }
  for (const value of Object.values(values)) {
    if (!Number.isSafeInteger(value) || value < 1 || value > maximum) {
      throw new Error("invalid_extension_contract");
    }
  }
  return Object.freeze({ ...values });
}

function strings(values, maximumCount = 128) {
  if (!Array.isArray(values) || values.length < 1 || values.length > maximumCount) {
    throw new Error("invalid_extension_contract");
  }
  if (
    values.some((value) => !validContractCode(value))
    || new Set(values).size !== values.length
  ) {
    throw new Error("invalid_extension_contract");
  }
  return Object.freeze([...values]);
}

function validContractCode(value) {
  if (typeof value !== "string" || value.length > LIMITS.maxContractCodeChars) return false;
  return /^[a-z][a-z0-9_.-]*$/.test(value);
}

export const API_VERSION = contract.apiVersion;
if (typeof API_VERSION !== "string" || !/^\d+$/.test(API_VERSION)) {
  throw new Error("invalid_extension_contract");
}
export const LIMITS = numericObject("limits");
export const TIMEOUTS = numericObject("timeouts");
if (
  TIMEOUTS.toolCallTimeoutMs >= TIMEOUTS.hostRequestTimeoutMs
  || TIMEOUTS.mcpToolTimeoutMs >= TIMEOUTS.coreRequestTimeoutMs
) {
  throw new Error("invalid_extension_contract");
}

export const CAPABILITIES = strings(contract.capabilities);
export const CORE_TO_HOST_METHODS = strings(contract?.methods?.coreToHost);
export const EVENTS = strings(contract.events);
export const LOAD_STAGES = strings(contract.loadStages);
export const EFFECT_CLASSES = strings(contract.effectClasses);
export const HOST_DIAGNOSTIC_CODES = strings(contract?.diagnostics?.hostCodes, 32);
export const RUNTIME_DIAGNOSTIC_CODES = strings(contract?.diagnostics?.runtimeCodes, 32);
export const BACKEND_ERROR_CODES = strings(contract?.errors?.backendCodes);
export const PROTOCOL_ERROR_REASONS = strings(contract?.errors?.protocolReasons);

const methodLevels = {};
const methodKinds = {};
const hostMethods = contract?.methods?.hostToCore;
if (!Array.isArray(hostMethods) || hostMethods.length < 1 || hostMethods.length > 64) {
  throw new Error("invalid_extension_contract");
}
for (const method of hostMethods) {
  if (
    !method
    || typeof method !== "object"
    || !validContractCode(method.name)
    || !["stable", "advanced"].includes(method.level)
    || !["request", "notification"].includes(method.kind)
    || method.name in methodLevels
  ) {
    throw new Error("invalid_extension_contract");
  }
  if (method.kind === "request") {
    if (
      !Number.isSafeInteger(method.rustBudgetMs)
      || method.rustBudgetMs < 0
      || (method.rustBudgetMs >= TIMEOUTS.coreRequestTimeoutMs && method.rustBudgetMs !== 0)
    ) {
      throw new Error("invalid_extension_contract");
    }
  } else if (method.rustBudgetMs !== null) {
    throw new Error("invalid_extension_contract");
  }
  methodLevels[method.name] = method.level;
  methodKinds[method.name] = method.kind;
}
export const HOST_TO_CORE_METHOD_LEVELS = Object.freeze(methodLevels);
export const HOST_TO_CORE_METHOD_KINDS = Object.freeze(methodKinds);
const notificationMethods = Object.entries(methodKinds)
  .filter(([, kind]) => kind === "notification")
  .map(([name]) => name);
if (notificationMethods.length !== 1) throw new Error("invalid_extension_contract");
export const HOST_LOAD_STAGE_METHOD = notificationMethods[0];

const diagnosticCodes = [...HOST_DIAGNOSTIC_CODES, ...RUNTIME_DIAGNOSTIC_CODES];
if (new Set(diagnosticCodes).size !== diagnosticCodes.length) {
  throw new Error("invalid_extension_contract");
}
export const DIAGNOSTIC = Object.freeze(
  Object.fromEntries(diagnosticCodes.map((code) => [code, code])),
);

export function supportsEvent(event) {
  return EVENTS.includes(event);
}

export function supportsEffect(effect) {
  return EFFECT_CLASSES.includes(effect);
}

export function methodLevel(method) {
  return HOST_TO_CORE_METHOD_LEVELS[method];
}

export function methodKind(method) {
  return HOST_TO_CORE_METHOD_KINDS[method];
}
