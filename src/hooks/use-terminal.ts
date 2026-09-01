import { useCallback, useEffect, useRef, useState } from "react";
import { homeDir } from "@tauri-apps/api/path";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import type { AgentSessionLoadState } from "./use-agent-sessions";
import type { ProjectLoadState } from "./use-projects";
import { clampTerminalHeight, TERMINAL_DEFAULT_HEIGHT } from "./terminal-layout";
import { useTerminalPersistence } from "./use-terminal-persistence";
import { closeTabInGroup as closeGroupTab, updateTab } from "./terminal-groups";
import {
  folderName,
  generateId,
  MAX_GROUPS,
  MAX_TABS_PER_GROUP,
  MAX_TOTAL_TABS,
  DEFAULT_GROUP_KEY,
  normalizeTerminalLabel,
} from "./terminal-types";
import type { TerminalGroup, TerminalTab } from "./terminal-types";

export type { TerminalGroup, TerminalTab };

interface TerminalOwnerState {
  validGroupKeys: string[];
  projectLoadState: ProjectLoadState;
  sessionLoadState: AgentSessionLoadState;
  defaultLabel?: string;
}

function addTabToGroups(
  groups: Map<string, TerminalGroup>,
  groupKey: string,
  tab: TerminalTab,
): Map<string, TerminalGroup> | null {
  const group = groups.get(groupKey) ?? { tabs: [], activeTabId: null };
  let totalTabs = 0;
  let durableGroups = 0;
  for (const value of groups.values()) {
    totalTabs += value.tabs.length;
    if (value.tabs.length > 0) durableGroups += 1;
  }
  const createsGroup = group.tabs.length === 0;
  if (group.tabs.length >= MAX_TABS_PER_GROUP || totalTabs >= MAX_TOTAL_TABS
    || (createsGroup && durableGroups >= MAX_GROUPS)) return null;
  const next = new Map(groups);
  next.set(groupKey, { tabs: [...group.tabs, tab], activeTabId: tab.id });
  return next;
}

export function useTerminal(
  groupKey: string,
  defaultCwd: string,
  { validGroupKeys, projectLoadState, sessionLoadState, defaultLabel }: TerminalOwnerState,
) {
  const [groups, setGroups] = useState<Map<string, TerminalGroup>>(new Map());
  const groupsRef = useRef(groups);
  const [panelHeight, setPanelHeight] = useState(TERMINAL_DEFAULT_HEIGHT);
  const [resolvedCwd, setResolvedCwd] = useState(defaultCwd);
  const maxHeightRef = useRef(0);
  const legacyNoticeShownRef = useRef(false);
  const ownersReady = projectLoadState === "ready" && sessionLoadState === "ready";
  const { loaded, persistenceStatus } = useTerminalPersistence({
    groups,
    setGroups,
    canSave: ownersReady,
  });

  useEffect(() => { groupsRef.current = groups; }, [groups]);

  useEffect(() => {
    if (defaultCwd) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- prop change resets the fallback cwd
      setResolvedCwd(defaultCwd);
    } else {
      void homeDir().then(setResolvedCwd).catch(() => {
        // homeDir ne sert qu'au libellé ; « Terminal » reste le repli sûr.
      });
    }
  }, [defaultCwd]);

  useEffect(() => {
    if (!loaded || !ownersReady) return;
    const valid = new Set(validGroupKeys);
    if (groupsRef.current.has(DEFAULT_GROUP_KEY) && !valid.has(DEFAULT_GROUP_KEY)
      && !legacyNoticeShownRef.current) {
      legacyNoticeShownRef.current = true;
      showToast(i18n.t("terminal.legacyTabsRemoved"), "info");
    }
    // eslint-disable-next-line react-hooks/set-state-in-effect -- readiness authorizes one cleanup pass
    setGroups((previous) => {
      if ([...previous.keys()].every((key) => valid.has(key))) return previous;
      const next = new Map(previous);
      for (const key of next.keys()) {
        if (!valid.has(key)) next.delete(key);
      }
      return next;
    });
  }, [loaded, ownersReady, setGroups, validGroupKeys]);

  const currentGroup = groups.get(groupKey) ?? { tabs: [], activeTabId: null };

  const allTabs = useCallback((): { tab: TerminalTab; groupKey: string }[] => {
    const result: { tab: TerminalTab; groupKey: string }[] = [];
    for (const [key, group] of groups) {
      for (const tab of group.tabs) result.push({ tab, groupKey: key });
    }
    return result;
  }, [groups]);

  const mutateGroups = useCallback((mutation: (value: Map<string, TerminalGroup>) => Map<string, TerminalGroup>) => {
    // La projection éphémère ordonne les appels synchrones; React reste l'autorité durable.
    groupsRef.current = mutation(groupsRef.current);
    setGroups(mutation);
  }, []);

  const addTab = useCallback((cwd?: string): string | null => {
    if (!loaded || persistenceStatus !== "healthy") return null;
    const label = normalizeTerminalLabel(folderName(cwd || defaultLabel || resolvedCwd));
    if (!label) {
      showToast(i18n.t("terminal.tabLimitReached"), "error");
      return null;
    }
    const tab: TerminalTab = {
      id: generateId(),
      ptyId: null,
      ptyToken: null,
      label,
      hasActivity: false,
    };
    const projected = addTabToGroups(groupsRef.current, groupKey, tab);
    if (projected === null) {
      showToast(i18n.t("terminal.tabLimitReached"), "error");
      return null;
    }
    /* Cette projection ne commite rien : elle sérialise seulement les admissions
       synchrones. React reste l'autorité et recalcule depuis son état précédent. */
    groupsRef.current = projected;
    setGroups((previous) => addTabToGroups(previous, groupKey, tab) ?? previous);
    return tab.id;
  }, [defaultLabel, groupKey, loaded, persistenceStatus, resolvedCwd]);

  const closeTabInGroup = useCallback((key: string, id: string): void => {
    mutateGroups((previous) => closeGroupTab(previous, key, id).groups);
  }, [mutateGroups]);

  const closeTab = useCallback((id: string): void => {
    closeTabInGroup(groupKey, id);
  }, [closeTabInGroup, groupKey]);

  const setActiveTab = useCallback((id: string) => {
    mutateGroups((previous) => {
      const group = previous.get(groupKey);
      if (!group || !group.tabs.some((tab) => tab.id === id)) return previous;
      const next = new Map(previous);
      next.set(groupKey, { ...group, activeTabId: id });
      return next;
    });
  }, [groupKey, mutateGroups]);

  const renameTab = useCallback((id: string, value: string): boolean => {
    const label = normalizeTerminalLabel(value);
    const group = groupsRef.current.get(groupKey);
    if (!label || !group?.tabs.some((tab) => tab.id === id)) return false;
    mutateGroups((previous) => {
      const current = previous.get(groupKey);
      if (!current) return previous;
      const next = new Map(previous);
      next.set(groupKey, {
        ...current,
        tabs: current.tabs.map((tab) => (tab.id === id ? { ...tab, label } : tab)),
      });
      return next;
    });
    return true;
  }, [groupKey, mutateGroups]);

  const reorderTabs = useCallback((fromIndex: number, toIndex: number) => {
    mutateGroups((previous) => {
      const group = previous.get(groupKey);
      if (!group || fromIndex < 0 || toIndex < 0 || fromIndex === toIndex
        || fromIndex >= group.tabs.length || toIndex >= group.tabs.length) return previous;
      const tabs = [...group.tabs];
      const [moved] = tabs.splice(fromIndex, 1);
      if (!moved) return previous;
      tabs.splice(toIndex, 0, moved);
      const next = new Map(previous);
      next.set(groupKey, { ...group, tabs });
      return next;
    });
  }, [groupKey, mutateGroups]);

  const setPtyId = useCallback((tabId: string, ptyId: number, ptyToken?: string) => {
    mutateGroups((previous) => updateTab(previous, tabId, { ptyId, ptyToken: ptyToken ?? null }) ?? previous);
  }, [mutateGroups]);

  const setTabActivity = useCallback((tabId: string, hasActivity: boolean) => {
    mutateGroups((previous) => updateTab(previous, tabId, { hasActivity }) ?? previous);
  }, [mutateGroups]);

  const resizePanel = useCallback((height: number) => {
    const clamped = clampTerminalHeight(height, maxHeightRef.current);
    setPanelHeight(clamped);
    return clamped;
  }, []);
  const setMaxHeight = useCallback((height: number) => { maxHeightRef.current = height; }, []);
  const removeGroup = useCallback((key: string) => {
    mutateGroups((previous) => {
      if (!previous.has(key)) return previous;
      const next = new Map(previous);
      next.delete(key);
      return next;
    });
  }, [mutateGroups]);
  const getGroupPtyIds = useCallback((key: string): number[] =>
    (groups.get(key)?.tabs ?? []).flatMap((tab) => tab.ptyId === null ? [] : [tab.ptyId]), [groups]);
  const getGroupPtyEntries = useCallback((key: string): { id: number; token: string }[] =>
    (groups.get(key)?.tabs ?? []).flatMap((tab) => tab.ptyId === null || tab.ptyToken === null
      ? [] : [{ id: tab.ptyId, token: tab.ptyToken }]), [groups]);

  return {
    tabs: currentGroup.tabs, activeTabId: currentGroup.activeTabId,
    panelHeight, loaded, persistenceStatus,
    allTabs, addTab, closeTab, closeTabInGroup, setActiveTab, renameTab, reorderTabs,
    setPtyId, setTabActivity, resizePanel, setMaxHeight, removeGroup,
    getGroupPtyIds, getGroupPtyEntries, groupKey,
  };
}
