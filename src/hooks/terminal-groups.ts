import type { TerminalGroup, TerminalTab } from "./terminal-types";

export function closeTabInGroup(
  groups: Map<string, TerminalGroup>,
  groupKey: string,
  tabId: string,
): { groups: Map<string, TerminalGroup>; groupBecameEmpty: boolean; changed: boolean } {
  const group = groups.get(groupKey);
  const closedIndex = group?.tabs.findIndex(({ id }) => id === tabId) ?? -1;
  if (!group || closedIndex < 0) {
    return { groups, groupBecameEmpty: false, changed: false };
  }
  const tabs = group.tabs.filter(({ id }) => id !== tabId);
  const activeTabId = group.activeTabId === tabId
    ? tabs[Math.min(closedIndex, tabs.length - 1)]?.id ?? null
    : group.activeTabId;
  const next = new Map(groups);
  next.set(groupKey, { tabs, activeTabId });
  return { groups: next, groupBecameEmpty: tabs.length === 0, changed: true };
}

/**
 * Remplace un onglet là où il se trouve, sans que l'appelant ait à savoir dans
 * quel groupe il vit.
 *
 * Rend `null` quand rien ne change. Sans ce retour, la sortie d'un programme
 * bavard provoquerait un rendu — et une écriture sur le disque — par ligne
 * reçue, pour réécrire la valeur déjà en place.
 */
export function updateTab(
  groups: Map<string, TerminalGroup>,
  tabId: string,
  patch: Partial<TerminalTab>,
): Map<string, TerminalGroup> | null {
  for (const [key, group] of groups) {
    const tab = group.tabs.find((t) => t.id === tabId);
    if (!tab) continue;
    const unchanged = Object.entries(patch).every(
      ([field, value]) => tab[field as keyof TerminalTab] === value,
    );
    if (unchanged) return null;
    const next = new Map(groups);
    next.set(key, {
      ...group,
      tabs: group.tabs.map((t) => (t.id === tabId ? { ...t, ...patch } : t)),
    });
    return next;
  }
  return null;
}
