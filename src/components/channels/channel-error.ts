const KNOWN_ERRORS = new Set(["invalidConfig", "unavailable"]);

export function channelErrorKey(code: string): string {
  return KNOWN_ERRORS.has(code) ? `channels.errors.${code}` : "channels.errors.generic";
}
