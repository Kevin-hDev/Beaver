import { MAX_BROWSER_URL_LENGTH } from "./browser-contract.generated";

export {
  MAX_BROWSER_SURFACE_EDGE,
  MAX_BROWSER_TABS,
  MAX_BROWSER_URL_LENGTH,
  MAX_LOCAL_SITES,
} from "./browser-contract.generated";
export type {
  BrowserSessionState,
  BrowserTabState,
  LocalSite,
  LocalSiteScanResult as LocalSiteScan,
} from "./browser-contract.generated";

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
