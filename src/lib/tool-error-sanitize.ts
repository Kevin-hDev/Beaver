const MAX_MESSAGE_CHARS = 300;
const MAX_DETAILS_CHARS = 20_000;

export function sanitizeToolError(input: string): string {
  const firstLine = input.split(/\r?\n/).find((line) => line.trim().length > 0) ?? "";
  return truncate(redact(removeUnsafeControls(firstLine)), MAX_MESSAGE_CHARS);
}

export function sanitizeToolErrorDetails(input: string): string {
  return truncate(redact(removeUnsafeControls(input)), MAX_DETAILS_CHARS);
}

function redact(input: string): string {
  return input
    .replace(/(bearer\s+)[a-z0-9._~+/=-]{8,}/gi, "$1[redacted]")
    .replace(/(basic\s+)[a-z0-9+/=]{8,}/gi, "$1[redacted]")
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/)[^/\s@]+@/gi, "$1[redacted]@")
    .replace(
      /((?:api[_-]?key|secret[_-]?key|access[_-]?token|refresh[_-]?token|token|secret|password|authorization|proxy[_-]?authorization|cookie|set[_-]?cookie|credential|private[_-]?key)["']?\s*[:=]\s*)(["'])[^\r\n]*?\2/gi,
      "$1$2[redacted]$2",
    )
    .replace(
      /((?:api[_-]?key|secret[_-]?key|access[_-]?token|refresh[_-]?token|token|secret|password|authorization|proxy[_-]?authorization|cookie|set[_-]?cookie|credential|private[_-]?key)["']?\s*[:=]\s*)[^;,}\s]+/gi,
      "$1[redacted]",
    )
    .replace(/\beyJ[a-z0-9_-]{6,}\.[a-z0-9_-]{6,}\.[a-z0-9_-]{6,}\b/gi, "[redacted]")
    .replace(/\b(?:AKIA|ASIA)[A-Z0-9]{16}\b/g, "[redacted]")
    .replace(/\b(?:sk-[a-z0-9_-]{16,}|gh[pousr]_[a-z0-9]{20,}|AIza[a-z0-9_-]{20,}|xox[baprs]-[a-z0-9-]{16,})\b/gi, "[redacted]")
    .replace(/file:\/\/[^\s;]+/gi, "[path]")
    .replace(/(^|[\s("'`:=])\/(?!\/)[^\s;]+/gm, "$1[path]")
    .replace(/(^|[\s("'`:=])\.\.?\/[^\s;]+/gm, "$1[path]")
    .replace(/(^|[\s("'`:=])~[/\\][^\s;]+/gm, "$1[path]")
    .replace(/(^|[\s("'`:=])\\\\[^\s;]+/gm, "$1[path]")
    .replace(/[A-Z]:\\[^\s;]+/gi, "[path]")
    .replace(
      /(^|[\s("'`:=])([a-z0-9_.-]+[/\\][a-z0-9_./\\-]*[a-z0-9_.-])(?=$|[\s;,:\])])/gim,
      "$1[path]",
    );
}

function truncate(input: string, maxChars: number): string {
  const chars = [...input];
  if (chars.length <= maxChars) return input;
  return `${chars.slice(0, maxChars).join("")}...`;
}

function isUnsafeControl(character: string): boolean {
  const code = character.codePointAt(0) ?? 0;
  return (code < 32 && character !== "\n" && character !== "\t")
    || code === 127
    || code === 0x061c
    || (code >= 0x200b && code <= 0x200f)
    || (code >= 0x202a && code <= 0x202e)
    || code === 0x2060
    || (code >= 0x2066 && code <= 0x2069)
    || code === 0xfeff;
}

function removeUnsafeControls(input: string): string {
  return [...input].filter((character) => !isUnsafeControl(character)).join("");
}
