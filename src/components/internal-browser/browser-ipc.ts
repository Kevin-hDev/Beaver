import { invoke } from "@tauri-apps/api/core";
import type {
  BrowserSessionState,
  BrowserTabCreation,
  LocalSiteScanResult,
} from "./browser-contract.generated";
import type { LocalSiteScan } from "./browser-types";

async function sessionCommand(command: string, args: Record<string, unknown>) {
  return invoke<BrowserSessionState>(command, args);
}

export function openBrowserSession(conversationId: string): Promise<BrowserSessionState> {
  return sessionCommand("browser_open_session", { conversationId });
}

export async function createBrowserTab(
  conversationId: string,
  replaceTabId: string | null,
): Promise<BrowserTabCreation> {
  return invoke<BrowserTabCreation>("browser_create_tab", {
    conversationId,
    replaceTabId,
  });
}

export function activateBrowserTab(conversationId: string, tabId: string) {
  return sessionCommand("browser_activate_tab", { conversationId, tabId });
}

export function reorderBrowserTabs(
  conversationId: string,
  tabIds: string[],
): Promise<BrowserSessionState> {
  return sessionCommand("browser_reorder_tabs", { conversationId, tabIds });
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
  return invoke<LocalSiteScanResult>("browser_detect_local_sites", {
    homeVisible,
  });
}
