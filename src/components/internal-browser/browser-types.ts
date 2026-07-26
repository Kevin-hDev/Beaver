export const MAX_BROWSER_TABS = 10;
export const MAX_BROWSER_URL_LENGTH = 2_048;
export const MAX_BROWSER_SURFACE_EDGE = 16_384;
export const MAX_LOCAL_SITES = 32;

export interface BrowserTabState {
  id: string;
  title: string;
  url: string | null;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  released: boolean;
}

export interface BrowserSessionState {
  tabs: BrowserTabState[];
  activeTabId: string;
  generation: number;
}

export interface LocalSite {
  url: string;
  title: string;
  port: number;
  protocol: "http" | "https";
}

export interface LocalSiteScan {
  sites: LocalSite[];
  generation: number;
  changed: boolean;
}

export interface BrowserSurfaceBounds {
  x: number;
  y: number;
  width: number;
  height: number;
  visible: boolean;
  generation: number;
}

export interface BrowserSurfaceRequest {
  conversationId: string;
  tabId: string;
  url: string | null;
  bounds: BrowserSurfaceBounds;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function hasAtMostCharacters(value: string, limit: number): boolean {
  let count = 0;
  for (const _character of value) {
    count += 1;
    if (count > limit) return false;
  }
  return true;
}

export function isBrowserTabId(value: string): boolean {
  return /^[0-9a-f]{32}$/u.test(value);
}

function byteLength(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

function containsControlCharacter(value: string): boolean {
  for (const character of value) {
    const codePoint = character.codePointAt(0);
    if (codePoint !== undefined && (codePoint <= 31 || codePoint === 127)) return true;
  }
  return false;
}

export function normalizeBrowserUrl(input: string): string | null {
  if (
    !input || input.trim() !== input || input.includes("\\") ||
    byteLength(input) > MAX_BROWSER_URL_LENGTH || containsControlCharacter(input)
  ) return null;
  try {
    const parsed = new URL(input);
    if (
      !["http:", "https:"].includes(parsed.protocol) || !parsed.hostname ||
      parsed.username || parsed.password
    ) return null;
    const normalized = parsed.toString();
    return byteLength(normalized) <= MAX_BROWSER_URL_LENGTH ? normalized : null;
  } catch {
    return null;
  }
}

function parseTab(value: unknown): BrowserTabState | null {
  if (!isRecord(value)) return null;
  const { id, title, url, loading, canGoBack, canGoForward, released } = value;
  if (
    typeof id !== "string" || !isBrowserTabId(id) ||
    typeof title !== "string" || !hasAtMostCharacters(title, 80) ||
    typeof loading !== "boolean" || typeof canGoBack !== "boolean" ||
    typeof canGoForward !== "boolean" || typeof released !== "boolean"
  ) return null;
  const normalizedUrl = typeof url === "string" ? normalizeBrowserUrl(url) : null;
  if (url !== null && normalizedUrl === null) return null;
  return { id, title, url: normalizedUrl, loading, canGoBack, canGoForward, released };
}

export function parseBrowserSession(value: unknown): BrowserSessionState | null {
  if (!isRecord(value) || !Array.isArray(value.tabs)) return null;
  if (value.tabs.length < 1 || value.tabs.length > MAX_BROWSER_TABS) return null;
  if (
    typeof value.activeTabId !== "string" ||
    !Number.isSafeInteger(value.generation) || Number(value.generation) < 1
  ) return null;
  const tabs: BrowserTabState[] = [];
  const identifiers = new Set<string>();
  for (const rawTab of value.tabs) {
    const tab = parseTab(rawTab);
    if (!tab || identifiers.has(tab.id)) return null;
    identifiers.add(tab.id);
    tabs.push(tab);
  }
  if (!identifiers.has(value.activeTabId)) return null;
  return { tabs, activeTabId: value.activeTabId, generation: Number(value.generation) };
}

function parseLocalSite(value: unknown): LocalSite | null {
  if (!isRecord(value)) return null;
  const { url, title, port, protocol } = value;
  if (
    typeof url !== "string" || typeof title !== "string" ||
    !hasAtMostCharacters(title, 80) || !Number.isInteger(port) ||
    Number(port) < 1 || Number(port) > 65_535 ||
    (protocol !== "http" && protocol !== "https")
  ) return null;
  const normalized = normalizeBrowserUrl(url);
  if (!normalized) return null;
  const parsed = new URL(normalized);
  const hostname = parsed.hostname.toLowerCase();
  if (!["localhost", "127.0.0.1", "[::1]", "::1"].includes(hostname)) return null;
  if (parsed.protocol !== `${protocol}:` || Number(parsed.port) !== Number(port)) return null;
  return { url: normalized, title, port: Number(port), protocol };
}

export function parseLocalSiteScan(value: unknown): LocalSiteScan | null {
  if (!isRecord(value) || !Array.isArray(value.sites)) return null;
  if (
    value.sites.length > MAX_LOCAL_SITES || typeof value.changed !== "boolean" ||
    !Number.isSafeInteger(value.generation) || Number(value.generation) < 0
  ) return null;
  const sites: LocalSite[] = [];
  let previousPort = 0;
  for (const rawSite of value.sites) {
    const site = parseLocalSite(rawSite);
    if (!site || site.port <= previousPort) return null;
    previousPort = site.port;
    sites.push(site);
  }
  return { sites, generation: Number(value.generation), changed: value.changed };
}
