import type { TFunction } from "i18next";
import type { WakeupRun, WakeupRunErrorCode } from "@/types/wakeup";

export const WAKEUP_RUN_ERROR_CODES = [
  "failed",
  "rate_limited",
  "authentication_failed",
  "ollama_unavailable",
  "missed_unavailable",
  "scheduler_stopping",
  "capacity_reached",
] as const satisfies readonly WakeupRunErrorCode[];

const ERROR_KEYS: Record<WakeupRunErrorCode, string> = {
  failed: "failed",
  rate_limited: "rateLimited",
  authentication_failed: "authenticationFailed",
  ollama_unavailable: "ollamaUnavailable",
  missed_unavailable: "missedUnavailable",
  scheduler_stopping: "schedulerStopping",
  capacity_reached: "capacityReached",
};

const KNOWN_CODES = new Set<string>(WAKEUP_RUN_ERROR_CODES);

export function wakeupRunErrorMessage(
  run: Pick<WakeupRun, "error_code" | "error">,
  t: TFunction,
): string {
  const code = run.error_code;
  if (code && KNOWN_CODES.has(code)) {
    return t(`heartbeat.history.errors.${ERROR_KEYS[code]}`);
  }
  return run.error ? t("heartbeat.history.errors.failed") : "";
}
