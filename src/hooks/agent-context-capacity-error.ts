import i18n from "@/i18n";
import type { ContextCapacityDetails } from "@/types/agent";

const ERROR_CODE = "context_capacity_exceeded";
const MAX_SAFE_TOKENS = 16_777_216;

export function contextCapacityErrorMessage(
  code: string,
  details?: ContextCapacityDetails,
): string | null {
  if (code !== ERROR_CODE || !details || !validDetails(details)) return null;
  const key = details.requiredReportTokens > 0
    ? "errors.contextCapacityExceededWithReports"
    : "errors.contextCapacityExceeded";
  return i18n.t(key, {
    systemTokens: details.systemTokens,
    reportTokens: details.requiredReportTokens,
    toolTokens: details.toolTokens,
    requiredTokens: details.requiredTokens,
    maxInputTokens: details.maxInputTokens,
    contextWindow: details.contextWindow,
  });
}

function validDetails(details: ContextCapacityDetails): boolean {
  const values = [
    details.systemTokens,
    details.requiredReportTokens,
    details.toolTokens,
    details.requiredTokens,
    details.maxInputTokens,
    details.contextWindow,
  ];
  return values.every((value) => Number.isSafeInteger(value) && value >= 0
      && value <= MAX_SAFE_TOKENS)
    && details.contextWindow > 0
    && details.maxInputTokens <= details.contextWindow
    && details.requiredTokens === details.systemTokens
      + details.requiredReportTokens + details.toolTokens
    && details.requiredTokens > details.maxInputTokens;
}
