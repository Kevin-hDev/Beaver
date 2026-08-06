import localStoreErrorKeys from "./local-store-error-contract.json";

type ErrorTranslator = (key: string) => string;

const LOCAL_STORE_ERROR_KEYS: Record<string, string> = localStoreErrorKeys;

export function localStoreErrorMessage(
  error: unknown,
  translate: ErrorTranslator,
): string {
  const key = typeof error === "string" ? LOCAL_STORE_ERROR_KEYS[error] : undefined;
  return translate(key ?? "errors.operationFailed");
}
