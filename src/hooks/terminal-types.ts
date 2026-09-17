import { MAX_LABEL_BYTES } from "@/types/terminal-contract.generated";

export {
  MAX_GROUP_KEY_BYTES,
  MAX_GROUPS,
  MAX_LABEL_BYTES,
  MAX_LIVE_TERMINALS,
  MAX_TABS_PER_GROUP,
  MAX_TOTAL_TABS,
} from "@/types/terminal-contract.generated";

export interface TerminalTab {
  id: string;
  ptyId: number | null;
  ptyToken: string | null;
  label: string;
  /** Du texte est arrivé pendant que l'onglet était en arrière-plan. Effacé
   *  dès qu'on l'ouvre, et jamais persisté : il ne décrit que cette session.
   *  Toujours présent, jamais absent — updateTab compare la valeur en place
   *  pour ne rien réécrire, et « absent » ne se compare à rien. */
  hasActivity: boolean;
}

export interface TerminalGroup {
  tabs: TerminalTab[];
  activeTabId: string | null;
}

export const DEFAULT_GROUP_KEY = "__default__";
/** Miroir frontend du plafond d'admission fixé par le gestionnaire PTY Rust. */
export function isValidTerminalLabel(value: unknown): value is string {
  return typeof value === "string"
    && value.length > 0
    && !/[\0\r\n]/u.test(value)
    && new TextEncoder().encode(value).length <= MAX_LABEL_BYTES;
}

export function normalizeTerminalLabel(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim();
  return isValidTerminalLabel(normalized) ? normalized : null;
}

export function generateId(): string {
  return crypto.randomUUID();
}

export function folderName(cwd: string): string {
  const parts = cwd.replace(/[\\/]$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || "Terminal";
}
