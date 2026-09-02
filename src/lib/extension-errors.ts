import { EXTENSION_BACKEND_ERROR_CODES } from "@/types/extension-contract.generated";

export { EXTENSION_BACKEND_ERROR_CODES };
const BACKEND_CODES: ReadonlySet<string> = new Set(EXTENSION_BACKEND_ERROR_CODES);

export function extensionErrorKey(error: unknown, fallback: string) {
  const code = typeof error === "string"
    ? error
    : error instanceof Error
      ? error.message
      : "";
  return BACKEND_CODES.has(code)
    ? `extensions.errors.codes.${code}`
    : fallback;
}
