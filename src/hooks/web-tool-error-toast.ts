import i18n from "@/i18n";
import { sanitizeToolError } from "@/lib/tool-error-sanitize";
import type { StreamEvent } from "@/types/agent";

const MAX_KEYS = 128;
const TOASTED_WEB_TOOLS = new Set(["web_search"]);
const shownKeys: string[] = [];
const shown = new Set<string>();

export function webToolErrorToastMessage(sessionId: string, event: StreamEvent): string | null {
  if (event.event !== "toolResult" || !event.data.isError) return null;
  if (!TOASTED_WEB_TOOLS.has(event.data.name)) return null;

  const detail = sanitizeToolError(event.data.content);

  const key = `${sessionId}:${event.data.name}:${event.data.toolCallIndex}`;
  if (shown.has(key)) return null;
  remember(key);

  return i18n.t("errors.webToolFailed", { message: detail || i18n.t("errors.toolFailed") });
}

function remember(key: string) {
  shown.add(key);
  shownKeys.push(key);
  while (shownKeys.length > MAX_KEYS) {
    const old = shownKeys.shift();
    if (old) shown.delete(old);
  }
}

export const sanitizeWebToolError = sanitizeToolError;

export function resetWebToolToastDedupeForTest() {
  shown.clear();
  shownKeys.length = 0;
}
