import { invoke } from "@tauri-apps/api/core";
import { parseTabCreation, type BrowserTabCreation } from "./browser-events";
import {
  parseBrowserSession,
  parseLocalSiteScan,
  type BrowserSessionState,
  type LocalSiteScan,
} from "./browser-types";

async function sessionCommand(command: string, args: Record<string, unknown>) {
  const session = parseBrowserSession(await invoke<unknown>(command, args));
  if (!session) throw new Error("invalid_browser_response");
  return session;
}

export function openBrowserSession(conversationId: string): Promise<BrowserSessionState> {
  return sessionCommand("browser_open_session", { conversationId });
}

export async function createBrowserTab(
  conversationId: string,
  replaceTabId: string | null,
): Promise<BrowserTabCreation> {
  const result = parseTabCreation(await invoke<unknown>("browser_create_tab", {
    conversationId,
    replaceTabId,
  }));
  if (!result) throw new Error("invalid_browser_response");
  return result;
}

export function activateBrowserTab(conversationId: string, tabId: string) {
  return sessionCommand("browser_activate_tab", { conversationId, tabId });
}

export function closeBrowserTab(conversationId: string, tabId: string) {
  return sessionCommand("browser_close_tab", { conversationId, tabId });
}

export function navigateBrowserTab(conversationId: string, tabId: string, url: string) {
  return sessionCommand("browser_navigate", { conversationId, tabId, url });
}

export async function runBrowserNavigationAction(
  conversationId: string,
  tabId: string,
  action: "back" | "forward" | "reloadOrStop",
) {
  await invoke("browser_navigation_action", { conversationId, tabId, action });
}

export async function detectLocalSites(homeVisible: boolean): Promise<LocalSiteScan> {
  const result = parseLocalSiteScan(await invoke<unknown>("browser_detect_local_sites", {
    homeVisible,
  }));
  if (!result) throw new Error("invalid_browser_response");
  return result;
}
