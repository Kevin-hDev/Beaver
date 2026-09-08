import { isBrowserTabId } from "./browser-types";
import { FAVICON_EVENT_VERSION, MAX_FAVICONS, MAX_FAVICON_PNG_BYTES, type BrowserFaviconSnapshot } from "./browser-favicon-contract";

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isPng(value: unknown): value is string {
  if (typeof value !== "string" || value.length > Math.ceil(MAX_FAVICON_PNG_BYTES / 3) * 4
      || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(value)) return false;
  try {
    const bytes = atob(value);
    return bytes.length <= MAX_FAVICON_PNG_BYTES && bytes.startsWith("\x89PNG\r\n\x1a\n") && btoa(bytes) === value;
  } catch {
    return false;
  }
}

export function parseFaviconSnapshot(value: unknown, conversationId: string): BrowserFaviconSnapshot | null {
  if (!isRecord(value) || value.eventVersion !== FAVICON_EVENT_VERSION || value.conversationId !== conversationId
      || typeof value.revision !== "number" || !Number.isSafeInteger(value.revision) || value.revision < 0
      || !Array.isArray(value.icons) || value.icons.length > MAX_FAVICONS) return null;
  const seen = new Set<string>();
  const icons: BrowserFaviconSnapshot["icons"] = [];
  for (const icon of value.icons) {
    if (!isRecord(icon) || typeof icon.tabId !== "string" || !isBrowserTabId(icon.tabId)
        || seen.has(icon.tabId) || !isPng(icon.pngBase64)) return null;
    seen.add(icon.tabId);
    icons.push({ tabId: icon.tabId, pngBase64: icon.pngBase64 });
  }
  return { eventVersion: FAVICON_EVENT_VERSION, revision: value.revision, conversationId, icons };
}
