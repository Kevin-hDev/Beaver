import { invoke } from "@tauri-apps/api/core";
import type { TerminalTabsDocument } from "@/types/terminal-contract.generated";

export type { TerminalSavedTab, TerminalTabsDocument } from "@/types/terminal-contract.generated";

export const TERMINAL_TABS_RECOVERED = "terminal-tabs-recovered";

export async function loadSavedGroups(): Promise<TerminalTabsDocument> {
  return invoke<TerminalTabsDocument>("load_terminal_tabs");
}

export const saveGroups = (document: TerminalTabsDocument): Promise<void> =>
  invoke<void>("save_terminal_tabs", { document });
