export const FILE_PREVIEW_MIN_WIDTH = 250;
export const FILE_PREVIEW_DEFAULT_WIDTH = 360;
export const FILE_PREVIEW_DEFAULT_EXTRA_WIDTH = 0;
export const CHAT_MIN_WIDTH = 360;

const MAX_STORED_TABS = 6;

export interface StoredFilePreviewPanel {
  open: boolean;
  fullscreen: boolean;
  width: number;
}

function tabsStorageKey(sessionId: string | null): string {
  return `clgo-file-preview-tabs:${sessionId ?? "none"}`;
}

function panelStorageKey(sessionId: string | null): string {
  return `clgo-file-preview-panel:${sessionId ?? "none"}`;
}

function clampFilePreviewWidth(value: unknown): number {
  const maxWidth = typeof window === "undefined" ? 1600 : Math.max(FILE_PREVIEW_MIN_WIDTH, window.innerWidth - CHAT_MIN_WIDTH);
  const width = typeof value === "number" && Number.isFinite(value) ? value : FILE_PREVIEW_DEFAULT_WIDTH;
  return Math.min(maxWidth, Math.max(FILE_PREVIEW_MIN_WIDTH, width));
}

export function clampFilePreviewWidthForContainer(
  value: unknown,
  containerWidth: number,
  reservedWidth = 0,
  chatMinWidth = CHAT_MIN_WIDTH,
): number {
  const safeChatMinWidth = Math.max(0, chatMinWidth);
  const panelSpace = Math.max(0, containerWidth - Math.max(0, reservedWidth));
  const targetChatMaxWidth = Math.max(0, panelSpace - safeChatMinWidth);
  const maxWidth = Math.max(Math.min(FILE_PREVIEW_MIN_WIDTH, panelSpace), targetChatMaxWidth);
  const minWidth = Math.min(FILE_PREVIEW_MIN_WIDTH, panelSpace);
  const width = typeof value === "number" && Number.isFinite(value) ? value : FILE_PREVIEW_DEFAULT_WIDTH;
  return Math.min(maxWidth, Math.max(minWidth, width));
}

export function readStoredFilePreviewTabs(sessionId: string | null): string[] {
  try {
    const raw = localStorage.getItem(tabsStorageKey(sessionId));
    const parsed = JSON.parse(raw ?? "[]") as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((id) => typeof id === "string").slice(0, MAX_STORED_TABS);
  } catch {
    return [];
  }
}

export function writeStoredFilePreviewTabs(sessionId: string | null, tabIds: string[]) {
  localStorage.setItem(tabsStorageKey(sessionId), JSON.stringify(tabIds.slice(0, MAX_STORED_TABS)));
}

export function readStoredFilePreviewPanel(sessionId: string | null): StoredFilePreviewPanel {
  try {
    const parsed = JSON.parse(localStorage.getItem(panelStorageKey(sessionId)) ?? "{}") as unknown;
    if (!parsed || typeof parsed !== "object") throw new Error("invalid");
    const state = parsed as Partial<StoredFilePreviewPanel>;
    return {
      open: state.open === true,
      fullscreen: state.fullscreen === true,
      width: clampFilePreviewWidth(state.width),
    };
  } catch {
    return {
      open: false,
      fullscreen: false,
      width: FILE_PREVIEW_DEFAULT_WIDTH,
    };
  }
}

export function writeStoredFilePreviewPanel(sessionId: string | null, state: StoredFilePreviewPanel) {
  localStorage.setItem(panelStorageKey(sessionId), JSON.stringify(state));
}
