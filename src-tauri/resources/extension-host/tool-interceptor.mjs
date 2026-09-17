import { snapshotContribution } from "./contribution-snapshot.mjs";
import { LIMITS } from "./contract.mjs";

export function createToolInterceptor(enabled) {
  let handler;

  function register(candidate) {
    if (!enabled || typeof candidate !== "function" || handler) {
      throw new Error("invalid_interceptor");
    }
    handler = candidate;
    let active = true;
    return () => {
      if (active) handler = undefined;
      active = false;
    };
  }

  async function invoke(call) {
    if (!handler) return frozenDecision("invalid");
    let result;
    try {
      result = await handler(snapshotContribution(call));
    } catch {
      return frozenDecision("failed");
    }
    try {
      const decision = snapshotContribution(result);
      if (decision.decision === "continue" && Object.keys(decision).length === 1) {
        return frozenDecision("continue");
      }
      if (
        decision.decision === "deny"
        && Object.keys(decision).every((key) => ["decision", "reason"].includes(key))
        && (decision.reason === undefined || (
          typeof decision.reason === "string"
          && Array.from(decision.reason).length <= LIMITS.maxExtensionTextChars
        ))
      ) {
        return Object.freeze(Object.assign(Object.create(null), {
          decision: "deny",
          reason: typeof decision.reason === "string" ? decision.reason : undefined,
        }));
      }
    } catch {
      // Invalid or mutable results fail closed below.
    }
    return frozenDecision("invalid");
  }

  return Object.freeze({
    register,
    invoke,
    contributions: () => (handler ? Object.freeze([Object.freeze(Object.create(null))]) : Object.freeze([])),
  });
}

function frozenDecision(decision) {
  return Object.freeze(Object.assign(Object.create(null), { decision }));
}
