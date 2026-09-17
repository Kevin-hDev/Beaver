import {
  BROWSER_EVENT_VERSION,
  type BrowserSessionEvent,
  type BrowserSessionState,
  type BrowserTabEvent as BrowserTabEventPayload,
  type PopupRequestEvent,
} from "./browser-contract.generated";

export const BROWSER_SESSION_EVENT = "browser-tab-state-changed-v1";
export const BROWSER_POPUP_EVENT = "browser-popup-request-v1";
export const BROWSER_READY_EVENT = "browser-view-ready-v1";
export const BROWSER_ENGINE_STOPPED_EVENT = "browser-engine-stopped-v1";
export const BROWSER_BLOCKED_FEATURE_EVENT = "browser-feature-blocked-v1";
export { BROWSER_EVENT_VERSION } from "./browser-contract.generated";
export type { BrowserTabCreation } from "./browser-contract.generated";
export type BrowserPopupRequest = Omit<PopupRequestEvent, "eventVersion" | "conversationId">;
export type BrowserTabEvent = Omit<BrowserTabEventPayload, "eventVersion" | "conversationId">;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function parseSessionEvent(
  value: unknown,
  conversationId: string,
): BrowserSessionState | null {
  if (
    !isRecord(value) || value.eventVersion !== BROWSER_EVENT_VERSION ||
    value.conversationId !== conversationId
  ) return null;
  return (value as BrowserSessionEvent).session;
}

export function parsePopupEvent(
  value: unknown,
  conversationId: string,
): BrowserPopupRequest | null {
  if (
    !isRecord(value) || value.eventVersion !== BROWSER_EVENT_VERSION ||
    value.conversationId !== conversationId
  ) return null;
  const { generation, sourceTabId, url } = value as PopupRequestEvent;
  return { generation, sourceTabId, url };
}

export function parseBrowserTabEvent(
  value: unknown,
  conversationId: string,
): BrowserTabEvent | null {
  if (
    !isRecord(value) || value.eventVersion !== BROWSER_EVENT_VERSION ||
    value.conversationId !== conversationId
  ) return null;
  const { generation, tabId } = value as BrowserTabEventPayload;
  return { generation, tabId };
}
