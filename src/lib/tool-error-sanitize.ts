const MAX_MESSAGE_CHARS = 300;

export function sanitizeToolError(input: string): string {
  const firstLine = input.split(/\r?\n/).find(Boolean) ?? "";
  return truncate(firstLine)
    .replace(/(bearer\s+)[a-z0-9._~+/=-]{8,}/gi, "$1[redacted]")
    .replace(/(api[_-]?key|secret[_-]?key|token|secret|password)\s*[:=]\s*[^;\s]+/gi, "$1=[redacted]")
    .replace(/\/Users\/[^\s;]+/g, "[path]")
    .replace(/[A-Z]:\\[^\s;]+/g, "[path]");
}

function truncate(input: string): string {
  const chars = [...input];
  if (chars.length <= MAX_MESSAGE_CHARS) return input;
  return `${chars.slice(0, MAX_MESSAGE_CHARS).join("")}...`;
}
