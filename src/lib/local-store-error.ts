import localStoreErrorKeys from "./local-store-error-contract.json";

type ErrorTranslator = (key: string) => string;

const LOCAL_STORE_ERROR_KEYS: Record<string, string> = localStoreErrorKeys;

export function localStoreErrorMessage(
  error: unknown,
  translate: ErrorTranslator,
): string {
  const code = typeof error === "string"
    ? error
    : error instanceof Error
      ? error.message
      : undefined;
  const key = code ? LOCAL_STORE_ERROR_KEYS[code] : undefined;
  return translate(key ?? "errors.operationFailed");
}
