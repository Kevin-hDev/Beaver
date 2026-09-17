import { FAVICON_EVENT_VERSION, type BrowserFaviconSnapshot } from "./browser-favicon-contract";

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function parseFaviconSnapshot(value: unknown, conversationId: string): BrowserFaviconSnapshot | null {
  if (!isRecord(value) || value.eventVersion !== FAVICON_EVENT_VERSION || value.conversationId !== conversationId
  ) return null;
  return value as BrowserFaviconSnapshot;
}
