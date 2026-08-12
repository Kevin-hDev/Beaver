import type { TFunction } from "i18next";

export const ADMISSION_ERROR_CODES = [
  "app-shutting-down",
  "app-work-capacity-reached",
  "service-shutting-down",
  "service-work-capacity-reached",
  "gateway-shutting-down",
  "gateway-busy",
] as const;

type AdmissionErrorCode = typeof ADMISSION_ERROR_CODES[number];
type AdmissionErrorKey = typeof ADMISSION_ERROR_KEYS[AdmissionErrorCode];

const MAX_CODE_CHARS = 64;
const ADMISSION_ERROR_KEYS = {
  "app-shutting-down": "errors.admission.appShuttingDown",
  "app-work-capacity-reached": "errors.admission.appCapacity",
  "service-shutting-down": "errors.admission.serviceShuttingDown",
  "service-work-capacity-reached": "errors.admission.serviceCapacity",
  "gateway-shutting-down": "errors.admission.gatewayShuttingDown",
  "gateway-busy": "errors.admission.gatewayBusy",
} as const;
const KNOWN_CODES = new Set<string>(ADMISSION_ERROR_CODES);

export function admissionErrorKey(error: unknown): AdmissionErrorKey | null {
  const code = admissionErrorCode(error);
  return code ? ADMISSION_ERROR_KEYS[code] : null;
}

export function admissionErrorMessage(
  error: unknown,
  t: TFunction,
  fallbackKey = "errors.operationFailed",
): string {
  return t(admissionErrorKey(error) ?? fallbackKey);
}

export function isAdmissionError(error: unknown): boolean {
  return admissionErrorCode(error) !== null;
}

function admissionErrorCode(error: unknown): AdmissionErrorCode | null {
  if (error instanceof Error) return admissionErrorCode(error.message);
  if (typeof error === "object" && error !== null) {
    return admissionErrorCode((error as Record<string, unknown>).code);
  }
  if (typeof error !== "string" || error.length > MAX_CODE_CHARS) return null;
  if (KNOWN_CODES.has(error)) return error as AdmissionErrorCode;
  try {
    const parsed: unknown = JSON.parse(error);
    return typeof parsed === "string" && KNOWN_CODES.has(parsed)
      ? parsed as AdmissionErrorCode
      : null;
  } catch {
    return null;
  }
}
