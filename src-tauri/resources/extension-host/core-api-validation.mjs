import { CORE_API_METHODS, LIMITS } from "./contract.mjs";

export function validateCoreApiParams(methodName, input) {
  const method = CORE_API_METHODS[methodName];
  if (!method) throw new Error("core_method_unavailable");
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    throw new Error("core_request_failed");
  }
  const allowed = new Map(method.params.map((param) => [param.name, param]));
  if (Object.keys(input).some((name) => !allowed.has(name))) {
    throw new Error("core_request_failed");
  }
  const output = {};
  for (const param of method.params) {
    const value = input[param.name];
    if (value === undefined) {
      if (param.required) throw new Error("core_request_failed");
      continue;
    }
    if (!validType(value, param.type)) throw new Error("core_request_failed");
    if (
      param.limit
      && typeof value === "string"
      && Buffer.byteLength(value, "utf8") > LIMITS[param.limit]
    ) throw new Error("core_request_failed");
    output[param.name] = value;
  }
  return Object.freeze(output);
}

export function validateCoreApiResult(value) {
  let encoded;
  try {
    encoded = Buffer.from(JSON.stringify(value), "utf8");
  } catch {
    throw new Error("core_request_failed");
  }
  if (encoded.length > LIMITS.maxMessageBytes) throw new Error("core_request_failed");
  return value;
}

function validType(value, type) {
  if (type === "string") return typeof value === "string";
  if (type === "integer") return Number.isSafeInteger(value);
  if (type === "boolean") return typeof value === "boolean";
  if (type === "object") return value !== null && typeof value === "object" && !Array.isArray(value);
  if (type === "memoryScope") return value === "global" || value === "project";
  if (type === "subagentType") return value === "explorer" || value === "coder";
  return false;
}
