import { invoke } from "@tauri-apps/api/core";
import type { CloneSessionResult, SessionTabs } from "@/types/agent";

/**
 * Helpers purs pour {@link useSessionTabs}. Extraits dans un fichier dédié pour
 * respecter la règle projet (fichiers < 200 lignes) et la séparation logique métier
 * (ces fonctions n'ont pas d'état React).
 */

/** Retrouve l'id d'onglet du clone fraîchement créé dans le résultat d'IPC. */
export function findCloneTabId(result: CloneSessionResult): string | null {
  return result.tabs.tabs.find((tab) => tab.session_id === result.clone_session_id)?.tab_id ?? null;
}

/**
 * Persiste l'onglet actif précédent après un clone. Utilisé quand l'utilisateur a
 * quitté la session pendant la génération du résumé (le clone est créé mais ne doit
 * pas voler le focus).
 */
export async function savePreviousActiveTab(
  rootSessionId: string,
  tabs: SessionTabs,
  previousActiveTabId: string,
): Promise<SessionTabs> {
  const activeTabExists = tabs.tabs.some((tab) => tab.tab_id === previousActiveTabId);
  return invoke<SessionTabs>("save_session_tabs", {
    sessionId: rootSessionId,
    tabs: { ...tabs, active_tab_id: activeTabExists ? previousActiveTabId : "main" },
  });
}

/** Ajoute un tab à la liste d'attention d'une session (borne à 3 entrées). */
export function addAttentionTab(
  current: Record<string, string[]>,
  rootSessionId: string,
  tabId: string,
): Record<string, string[]> {
  const ids = current[rootSessionId] ?? [];
  if (ids.includes(tabId)) return current;
  return { ...current, [rootSessionId]: [...ids, tabId].slice(-3) };
}

/** Retire un tab de la liste d'attention d'une session. */
export function removeAttentionTab(
  current: Record<string, string[]>,
  rootSessionId: string,
  tabId: string,
): Record<string, string[]> {
  const ids = current[rootSessionId];
  if (!ids?.includes(tabId)) return current;
  const nextIds = ids.filter((id) => id !== tabId);
  const next = { ...current };
  if (nextIds.length > 0) next[rootSessionId] = nextIds;
  else delete next[rootSessionId];
  return next;
}
