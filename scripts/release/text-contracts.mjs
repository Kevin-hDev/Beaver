const MAX_TEXT_LENGTH = 2 * 1024 * 1024;

export function normalizeNewlines(value) {
  if (typeof value !== "string" || value.length > MAX_TEXT_LENGTH) {
    throw new Error("Release contract validation failed");
  }

  return value.replaceAll("\r\n", "\n").replaceAll("\r", "\n");
}
