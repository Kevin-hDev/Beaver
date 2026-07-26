import {
  hasAtMostCharacters,
  isBrowserTabId,
  normalizeBrowserUrl,
  parseBrowserSession,
  type BrowserSessionState,
} from "./browser-types";

export const BROWSER_SESSION_EVENT = "browser-tab-state-changed-v1";
export const BROWSER_POPUP_EVENT = "browser-popup-request-v1";
export const BROWSER_READY_EVENT = "browser-view-ready-v1";
export const BROWSER_ENGINE_STOPPED_EVENT = "browser-engine-stopped-v1";
export const BROWSER_BLOCKED_FEATURE_EVENT = "browser-feature-blocked-v1";
export const BROWSER_EVENT_VERSION = 1;

export type BrowserTabCreation =
  | { status: "created"; session: BrowserSessionState }
  | { status: "confirmationRequired"; candidateId: string; candidateTitle: string };

export interface BrowserPopupRequest {
  generation: number;
  sourceTabId: string;
  url: string;
}

export interface BrowserTabEvent {
  generation: number;
  tabId: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function parseTabCreation(value: unknown): BrowserTabCreation | null {
  if (!isRecord(value) || typeof value.status !== "string") return null;
  if (value.status === "created") {
    const session = parseBrowserSession(value.session);
    return session ? { status: "created", session } : null;
  }
  if (
    value.status !== "confirmationRequired" || typeof value.candidateId !== "string" ||
    !isBrowserTabId(value.candidateId) || typeof value.candidateTitle !== "string" ||
    !hasAtMostCharacters(value.candidateTitle, 80)
  ) return null;
  return {
    status: "confirmationRequired",
    candidateId: value.candidateId,
    candidateTitle: value.candidateTitle,
  };
}

export function parseSessionEvent(
  value: unknown,
  conversationId: string,
): BrowserSessionState | null {
  if (
    !isRecord(value) || value.eventVersion !== BROWSER_EVENT_VERSION ||
    value.conversationId !== conversationId
  ) return null;
  return parseBrowserSession(value.session);
}

export function parsePopupEvent(
  value: unknown,
  conversationId: string,
): BrowserPopupRequest | null {
  if (
    !isRecord(value) || value.eventVersion !== BROWSER_EVENT_VERSION ||
    value.conversationId !== conversationId ||
    !Number.isSafeInteger(value.generation) || Number(value.generation) < 1 ||
    typeof value.sourceTabId !== "string" || !isBrowserTabId(value.sourceTabId) ||
    typeof value.url !== "string"
  ) return null;
  const url = normalizeBrowserUrl(value.url);
  return url ? { generation: Number(value.generation), sourceTabId: value.sourceTabId, url } : null;
}

export function parseBrowserTabEvent(
  value: unknown,
  conversationId: string,
): BrowserTabEvent | null {
  if (
    !isRecord(value) || value.eventVersion !== BROWSER_EVENT_VERSION ||
    value.conversationId !== conversationId ||
    !Number.isSafeInteger(value.generation) || Number(value.generation) < 1 ||
    typeof value.tabId !== "string" || !isBrowserTabId(value.tabId)
  ) return null;
  return { generation: Number(value.generation), tabId: value.tabId };
}
