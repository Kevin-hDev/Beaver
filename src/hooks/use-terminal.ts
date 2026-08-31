import { useCallback, useEffect, useRef, useState } from "react";
import { homeDir } from "@tauri-apps/api/path";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import type { ProjectLoadState } from "./use-projects";
import { clampTerminalHeight, TERMINAL_DEFAULT_HEIGHT } from "./terminal-layout";
import { useTerminalPersistence } from "./use-terminal-persistence";
import { closeTabInGroup as closeGroupTab, updateTab } from "./terminal-groups";
import {
  DEFAULT_GROUP_KEY,
  folderName,
  generateId,
  normalizeTerminalLabel,
} from "./terminal-types";
import type { TerminalGroup, TerminalTab } from "./terminal-types";

export type { TerminalGroup, TerminalTab };

const MAX_GROUPS = 128;
const MAX_TABS_PER_GROUP = 16;
const MAX_TOTAL_TABS = 256;

interface TerminalProjectState {
  validGroupKeys: string[];
  projectLoadState: ProjectLoadState;
}

export function useTerminal(
  groupKey: string,
  defaultCwd: string,
  { validGroupKeys, projectLoadState }: TerminalProjectState,
) {
  const [groups, setGroups] = useState<Map<string, TerminalGroup>>(new Map());
  const groupsRef = useRef(groups);
  const [panelHeight, setPanelHeight] = useState(TERMINAL_DEFAULT_HEIGHT);
  const [resolvedCwd, setResolvedCwd] = useState(defaultCwd);
  const maxHeightRef = useRef(0);
  const { loaded, persistenceStatus } = useTerminalPersistence({
    groups,
    setGroups,
    canSave: projectLoadState === "ready",
  });

  useEffect(() => { groupsRef.current = groups; }, [groups]);

  useEffect(() => {
    if (defaultCwd) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- prop change resets the fallback cwd
      setResolvedCwd(defaultCwd);
    } else {
      void homeDir().then(setResolvedCwd).catch(() => {});
    }
  }, [defaultCwd]);

  useEffect(() => {
    if (!loaded || projectLoadState !== "ready") return;
    const valid = new Set([...validGroupKeys, DEFAULT_GROUP_KEY]);
    // eslint-disable-next-line react-hooks/set-state-in-effect -- readiness authorizes one cleanup pass
    setGroups((previous) => {
      if ([...previous.keys()].every((key) => valid.has(key))) return previous;
      const next = new Map(previous);
      for (const key of next.keys()) {
        if (!valid.has(key)) next.delete(key);
      }
      return next;
    });
  }, [loaded, projectLoadState, setGroups, validGroupKeys]);

  const currentGroup = groups.get(groupKey) ?? { tabs: [], activeTabId: null };

  const allTabs = useCallback((): { tab: TerminalTab; groupKey: string }[] => {
    const result: { tab: TerminalTab; groupKey: string }[] = [];
    for (const [key, group] of groups) {
      for (const tab of group.tabs) result.push({ tab, groupKey: key });
    }
    return result;
  }, [groups]);

  const addTab = useCallback((cwd?: string): string | null => {
    const previous = groupsRef.current;
    const group = previous.get(groupKey) ?? { tabs: [], activeTabId: null };
    let totalTabs = 0;
    let durableGroups = 0;
    for (const value of previous.values()) {
      totalTabs += value.tabs.length;
      if (value.tabs.length > 0) durableGroups += 1;
    }
    const createsGroup = group.tabs.length === 0;
    const label = normalizeTerminalLabel(folderName(cwd || resolvedCwd));
    if (!label || group.tabs.length >= MAX_TABS_PER_GROUP || totalTabs >= MAX_TOTAL_TABS
      || (createsGroup && durableGroups >= MAX_GROUPS)) {
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
    const next = new Map(previous);
    next.set(groupKey, { tabs: [...group.tabs, tab], activeTabId: tab.id });
    groupsRef.current = next;
    setGroups(next);
    return tab.id;
  }, [groupKey, resolvedCwd]);

  const closeTabInGroup = useCallback((key: string, id: string): void => {
    setGroups((previous) => closeGroupTab(previous, key, id).groups);
  }, []);

  const closeTab = useCallback((id: string): void => {
    closeTabInGroup(groupKey, id);
  }, [closeTabInGroup, groupKey]);

  const setActiveTab = useCallback((id: string) => {
    setGroups((previous) => {
      const group = previous.get(groupKey);
      if (!group || !group.tabs.some((tab) => tab.id === id)) return previous;
      const next = new Map(previous);
      next.set(groupKey, { ...group, activeTabId: id });
      return next;
    });
  }, [groupKey]);

  const renameTab = useCallback((id: string, value: string): boolean => {
    const label = normalizeTerminalLabel(value);
    const group = groupsRef.current.get(groupKey);
    if (!label || !group?.tabs.some((tab) => tab.id === id)) return false;
    setGroups((previous) => {
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
  }, [groupKey]);

  const reorderTabs = useCallback((fromIndex: number, toIndex: number) => {
    setGroups((previous) => {
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
  }, [groupKey]);

  const setPtyId = useCallback((tabId: string, ptyId: number, ptyToken?: string) => {
    setGroups((previous) => updateTab(previous, tabId, { ptyId, ptyToken: ptyToken ?? null }) ?? previous);
  }, []);

  const setTabActivity = useCallback((tabId: string, hasActivity: boolean) => {
    setGroups((previous) => updateTab(previous, tabId, { hasActivity }) ?? previous);
  }, []);

  const resizePanel = useCallback((height: number) => {
    const clamped = clampTerminalHeight(height, maxHeightRef.current);
    setPanelHeight(clamped);
    return clamped;
  }, []);
  const setMaxHeight = useCallback((height: number) => { maxHeightRef.current = height; }, []);
  const removeGroup = useCallback((key: string) => {
    setGroups((previous) => {
      if (!previous.has(key)) return previous;
      const next = new Map(previous);
      next.delete(key);
      return next;
    });
  }, []);
  const getGroupPtyIds = useCallback((key: string): number[] =>
    (groups.get(key)?.tabs ?? []).flatMap((tab) => tab.ptyId === null ? [] : [tab.ptyId]), [groups]);
  const getGroupPtyEntries = useCallback((key: string): { id: number; token: string }[] =>
    (groups.get(key)?.tabs ?? []).flatMap((tab) => tab.ptyId === null || tab.ptyToken === null
      ? [] : [{ id: tab.ptyId, token: tab.ptyToken }]), [groups]);

  return {
    tabs: currentGroup.tabs, activeTabId: currentGroup.activeTabId,
    panelHeight, persistenceStatus,
    allTabs, addTab, closeTab, closeTabInGroup, setActiveTab, renameTab, reorderTabs,
    setPtyId, setTabActivity, resizePanel, setMaxHeight, removeGroup,
    getGroupPtyIds, getGroupPtyEntries, groupKey,
  };
}
