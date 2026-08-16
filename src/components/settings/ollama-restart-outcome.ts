import type { OllamaStartOutcome } from "@/types/ollama-runtime";

export type OllamaRestartPresentation =
  | { kind: "owned" }
  | { kind: "external" }
  | { kind: "failed"; code: unknown };

export function classifyOllamaRestartOutcome(
  value: unknown,
): OllamaRestartPresentation {
  if (value === "rejected_during_shutdown" || !isRecord(value)) {
    return { kind: "failed", code: null };
  }
  if (hasEndpoint(value, "owned_started") || hasEndpoint(value, "owned_already_running")) {
    return { kind: "owned" };
  }
  if (hasEndpoint(value, "external_available")) return { kind: "external" };
  if (isRecord(value.blocked_by_recovery)) {
    return { kind: "failed", code: value.blocked_by_recovery.code };
  }
  if (isRecord(value.failed)) return { kind: "failed", code: value.failed.code };
  return { kind: "failed", code: null };
}

function hasEndpoint(
  value: Record<string, unknown>,
  key: "owned_started" | "owned_already_running" | "external_available",
): value is Extract<OllamaStartOutcome, Record<typeof key, unknown>> {
  if (!Object.prototype.hasOwnProperty.call(value, key)) return false;
  const payload = value[key];
  if (!isRecord(payload) || !isRecord(payload.endpoint)) return false;
  return Number.isInteger(payload.endpoint.port)
    && Number(payload.endpoint.port) > 0
    && Number(payload.endpoint.port) <= 65_535;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
