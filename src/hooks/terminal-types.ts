export interface TerminalTab {
  id: string;
  ptyId: number | null;
  ptyToken: string | null;
  label: string;
  cwd: string;
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

export function generateId(): string {
  return crypto.randomUUID();
}

export function folderName(cwd: string): string {
  const parts = cwd.replace(/[\\/]$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || "Terminal";
}
