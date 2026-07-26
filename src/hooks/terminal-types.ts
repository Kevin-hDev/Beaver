export interface TerminalTab {
  id: string;
  ptyId: number | null;
  ptyToken: string | null;
  label: string;
  cwd: string;
}

export interface TerminalGroup {
  tabs: TerminalTab[];
  activeTabId: string | null;
}

export const DEFAULT_GROUP_KEY = "__default__";

export function generateId(): string {
  return crypto.randomUUID();
}

export function folderName(cwd: string): string {
  const parts = cwd.replace(/[\\/]$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || "Terminal";
}
