import { AsyncLocalStorage } from "node:async_hooks";
import { LIMITS, TIMEOUTS } from "./contract.mjs";

const storage = new AsyncLocalStorage();
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu;
const SECRET = /^[0-9a-f]{64}$/u;

export function runWithCoreContext(input, callback) {
  const scope = validate(input);
  return storage.run(scope, callback);
}

export function coreContextForTransport() {
  const scope = storage.getStore();
  if (!scope) return undefined;
  const remainingMs = Math.max(0, Math.floor(scope.deadline - performance.now()));
  if (remainingMs === 0) throw new Error("core_context_expired");
  return Object.freeze({ id: scope.id, secret: scope.secret, remainingMs });
}

export function coreContextTimeout(maximumMs) {
  const scope = storage.getStore();
  if (!scope) return maximumMs;
  return Math.min(maximumMs, Math.max(0, Math.floor(scope.deadline - performance.now())));
}

function validate(input) {
  if (
    !input
    || typeof input !== "object"
    || Array.isArray(input)
    || Object.keys(input).some((key) => !["id", "secret", "remainingMs"].includes(key))
    || typeof input.id !== "string"
    || !UUID.test(input.id)
    || typeof input.secret !== "string"
    || !SECRET.test(input.secret)
    || !Number.isSafeInteger(input.remainingMs)
    || input.remainingMs < 1
    || input.remainingMs > TIMEOUTS.toolCallTimeoutMs
    || Buffer.byteLength(input.secret, "utf8") > LIMITS.maxIdentifierChars
  ) throw new Error("invalid_tool_scope");
  return Object.freeze({
    id: input.id,
    secret: input.secret,
    deadline: performance.now() + input.remainingMs,
  });
}
