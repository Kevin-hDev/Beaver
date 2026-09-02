import { EXTENSION_BACKEND_ERROR_CODES } from "@/types/extension-contract.generated";
import extensionContract from "../../src-tauri/resources/extension-host/contract.json";

export { EXTENSION_BACKEND_ERROR_CODES };
const BACKEND_CODES: ReadonlySet<string> = new Set(EXTENSION_BACKEND_ERROR_CODES);
const DIAGNOSTIC_CODES: ReadonlySet<string> = new Set([
  ...extensionContract.diagnostics.hostCodes,
  ...extensionContract.diagnostics.runtimeCodes,
]);

export function extensionErrorKey(error: unknown, fallback: string) {
  const code = typeof error === "string"
    ? error
    : error instanceof Error
      ? error.message
      : "";
  if (BACKEND_CODES.has(code)) {
    return `extensions.errors.codes.${code}`;
  }
  return DIAGNOSTIC_CODES.has(code)
    ? `extensions.diagnostics.codes.${code}`
    : fallback;
}
