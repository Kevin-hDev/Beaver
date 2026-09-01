import { invoke } from "@tauri-apps/api/core";
import { isValidTerminalLabel } from "./terminal-types";

export interface TerminalSavedTab {
  label: string;
}

export interface TerminalTabsDocument {
  version: 1;
  groups: Record<string, TerminalSavedTab[]>;
}

const MAX_GROUPS = 128;
const MAX_TABS_PER_GROUP = 16;
const MAX_TOTAL_TABS = 256;
const MAX_GROUP_KEY_BYTES = 128;
export const TERMINAL_TABS_RECOVERED = "terminal-tabs-recovered";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isValidGroupKey(value: string): boolean {
  return value.length > 0
    && new TextEncoder().encode(value).length <= MAX_GROUP_KEY_BYTES
    && !/[\0\r\n]/u.test(value);
}

function isTerminalTabsDocument(value: unknown): value is TerminalTabsDocument {
  if (!isObject(value) || value.version !== 1 || !isObject(value.groups)) return false;
  const groups = Object.entries(value.groups);
  if (groups.length > MAX_GROUPS) return false;
  let totalTabs = 0;
  for (const [groupKey, tabs] of groups) {
    if (!isValidGroupKey(groupKey) || !Array.isArray(tabs) || tabs.length > MAX_TABS_PER_GROUP) {
      return false;
    }
    totalTabs += tabs.length;
    if (totalTabs > MAX_TOTAL_TABS) return false;
    if (tabs.some((tab) => !isObject(tab) || !isValidTerminalLabel(tab.label))) {
      return false;
    }
  }
  return true;
}

export async function loadSavedGroups(): Promise<TerminalTabsDocument> {
  const value: unknown = await invoke("load_terminal_tabs");
  if (!isTerminalTabsDocument(value)) throw new Error("terminal-tabs-invalid");
  return value;
}

export const saveGroups = (document: TerminalTabsDocument): Promise<void> =>
  invoke<void>("save_terminal_tabs", { document });
