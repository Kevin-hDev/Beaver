import { extensionErrorKey } from "./extension-errors";

const INSTALL_ERRORS: Readonly<Record<string, string>> = {
  "extension-install-invalid": "invalid",
  "extension-install-busy": "busy",
  "extension-install-unavailable": "unavailable",
  "extension-install-failed": "failed",
  "extension-install-insufficient-space": "insufficientSpace",
  "extension-install-recovery-unavailable": "recoveryUnavailable",
};
export function installJobErrorKey(error: unknown, fallback = "extensionInstalls.errors.action"): string {
  const code = typeof error === "string" ? error : error instanceof Error ? error.message : "";
  const suffix = Object.prototype.hasOwnProperty.call(INSTALL_ERRORS, code) ? INSTALL_ERRORS[code] : null;
  return suffix ? `extensionInstalls.errors.${suffix}` : extensionErrorKey(error, fallback);
}
