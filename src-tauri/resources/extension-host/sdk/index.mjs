export function defineExtension(extension) {
  return extension;
}

export function isBeaverExtensionError(error) {
  return error instanceof Error
    && error.name === "BeaverExtensionError"
    && Number.isSafeInteger(error.code)
    && typeof error.reason === "string"
    && typeof error.retryable === "boolean";
}
