type ErrorTranslator = (key: string) => string;

const LOCAL_STORE_ERROR_KEYS: Record<string, string> = {
  "ollama-custom-store-unavailable": "errors.localStore.ollamaCustomization",
  "ollama-native-prompt-store-unavailable": "errors.localStore.ollamaNativePrompts",
  "system-prompt-store-unavailable": "errors.localStore.systemPrompts",
  "ollama-custom-store-missing": "errors.localStore.ollamaCustomizationMissing",
  "ollama-native-prompt-store-missing": "errors.localStore.ollamaNativePromptsMissing",
  "system-prompt-store-missing": "errors.localStore.systemPromptsMissing",
  "ollama-custom-store-write": "errors.localStore.ollamaCustomizationWrite",
  "ollama-native-prompt-write": "errors.localStore.ollamaNativePromptsWrite",
  "system-prompt-store-write": "errors.localStore.systemPromptsWrite",
};

export function localStoreErrorMessage(
  error: unknown,
  translate: ErrorTranslator,
): string {
  const key = typeof error === "string" ? LOCAL_STORE_ERROR_KEYS[error] : undefined;
  return translate(key ?? "errors.operationFailed");
}
