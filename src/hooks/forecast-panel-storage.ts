const PANEL_STORAGE_PREFIX = "fc-panel-";
const PANEL_STORAGE_ORDER_KEY = "fc-panel-session-order";
const MAX_STORED_PANEL_SESSIONS = 32;
const MAX_ORDER_CHARS = 8_192;
const MAX_PANEL_STATE_CHARS = 4_096;
const SAFE_SESSION_ID = /^[A-Za-z0-9_-]{1,128}$/;

function storageKey(sessionId: string): string {
  return `${PANEL_STORAGE_PREFIX}${sessionId}`;
}

function loadOrder(): string[] {
  const raw = localStorage.getItem(PANEL_STORAGE_ORDER_KEY);
  if (!raw || raw.length > MAX_ORDER_CHARS) return [];
  try {
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((value): value is string => (
        typeof value === "string" && SAFE_SESSION_ID.test(value)
      ))
      .slice(-MAX_STORED_PANEL_SESSIONS);
  } catch {
    return [];
  }
}

export function loadForecastPanelValue(sessionId: string): unknown {
  if (!SAFE_SESSION_ID.test(sessionId)) return null;
  try {
    const raw = localStorage.getItem(storageKey(sessionId));
    return raw && raw.length <= MAX_PANEL_STATE_CHARS ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

export function saveForecastPanelValue(sessionId: string, value: unknown): void {
  if (!SAFE_SESSION_ID.test(sessionId)) return;
  const key = storageKey(sessionId);
  try {
    localStorage.setItem(key, JSON.stringify(value));
    const order = loadOrder().filter((id) => id !== sessionId);
    order.push(sessionId);
    const removed = order.splice(0, Math.max(0, order.length - MAX_STORED_PANEL_SESSIONS));
    removed.forEach((id) => localStorage.removeItem(storageKey(id)));
    localStorage.setItem(PANEL_STORAGE_ORDER_KEY, JSON.stringify(order));
  } catch {
    try {
      localStorage.removeItem(key);
    } catch {
      return;
    }
  }
}

export function withBoundedPanelState<T>(
  states: Record<string, T>,
  key: string,
  value: T,
): Record<string, T> {
  const entries = Object.entries(states).filter(([candidate]) => candidate !== key);
  entries.push([key, value]);
  return Object.fromEntries(entries.slice(-MAX_STORED_PANEL_SESSIONS));
}
