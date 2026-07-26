import { useState, useCallback, useRef, useEffect } from "react";
import { homeDir } from "@tauri-apps/api/path";
import { loadSavedGroups, saveGroups } from "./terminal-persistence";
import { generateId, folderName, DEFAULT_GROUP_KEY } from "./terminal-types";
import type { TerminalTab, TerminalGroup } from "./terminal-types";

export type { TerminalTab, TerminalGroup };

const DEFAULT_HEIGHT = 120;
const MIN_HEIGHT = 80;

export function useTerminal(groupKey: string, defaultCwd: string, validGroupKeys?: string[]) {
  const [groups, setGroups] = useState<Map<string, TerminalGroup>>(new Map());
  const [global, setGlobal] = useState({ isOpen: false, panelHeight: DEFAULT_HEIGHT });
  const [resolvedCwd, setResolvedCwd] = useState(defaultCwd);
  const [loaded, setLoaded] = useState(false);
  const maxHeightRef = useRef(0);

  useEffect(() => {
    if (defaultCwd) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- sync state reset on defaultCwd change is intentional
      setResolvedCwd(defaultCwd);
    } else {
      homeDir().then(setResolvedCwd).catch(() => {});
    }
  }, [defaultCwd]);

  useEffect(() => {
    void loadSavedGroups().then((saved) => {
      const map = new Map<string, TerminalGroup>();
      for (const [key, tabs] of Object.entries(saved)) {
        map.set(key, {
          tabs: tabs.map((t) => ({ id: generateId(), ptyId: null, ptyToken: null, label: t.label, cwd: t.cwd })),
          activeTabId: null,
        });
      }
      for (const [, group] of map) {
        if (group.tabs.length > 0) group.activeTabId = group.tabs[0].id;
      }
      if (validGroupKeys) {
        const validSet = new Set([...validGroupKeys, DEFAULT_GROUP_KEY]);
        for (const key of map.keys()) {
          if (!validSet.has(key)) {
            map.delete(key);
          }
        }
      }
      setGroups(map);
      setLoaded(true);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps -- load once on mount
  }, []);

  useEffect(() => {
    if (!loaded) return;
    void saveGroups(groups);
  }, [groups, loaded]);

  const currentGroup = groups.get(groupKey) ?? { tabs: [], activeTabId: null };

  const allTabs = useCallback((): { tab: TerminalTab; groupKey: string }[] => {
    const result: { tab: TerminalTab; groupKey: string }[] = [];
    for (const [key, group] of groups) {
      for (const tab of group.tabs) result.push({ tab, groupKey: key });
    }
    return result;
  }, [groups]);

  const addTab = useCallback((cwd?: string) => {
    const dir = cwd || resolvedCwd;
    const tab: TerminalTab = { id: generateId(), ptyId: null, ptyToken: null, label: folderName(dir), cwd: dir };
    setGroups((prev) => {
      const next = new Map(prev);
      const group = next.get(groupKey) ?? { tabs: [], activeTabId: null };
      next.set(groupKey, { tabs: [...group.tabs, tab], activeTabId: tab.id });
      return next;
    });
    setGlobal((prev) => ({ ...prev, isOpen: true }));
    return tab.id;
  }, [groupKey, resolvedCwd]);

  const closeTab = useCallback((id: string) => {
    setGroups((prev) => {
      const next = new Map(prev);
      const group = next.get(groupKey);
      if (!group) return prev;
      const filtered = group.tabs.filter((t) => t.id !== id);
      let nextActive = group.activeTabId;
      if (group.activeTabId === id) {
        const closedIdx = group.tabs.findIndex((t) => t.id === id);
        const nextTab = filtered[Math.min(closedIdx, filtered.length - 1)];
        nextActive = nextTab?.id ?? null;
      }
      next.set(groupKey, { tabs: filtered, activeTabId: nextActive });
      return next;
    });
    setGlobal((prev) => {
      const group = groups.get(groupKey);
      if (!group) return prev;
      if (group.tabs.filter((t) => t.id !== id).length === 0) return { ...prev, isOpen: false };
      return prev;
    });
  }, [groupKey, groups]);

  const setActiveTab = useCallback((id: string) => {
    setGroups((prev) => {
      const next = new Map(prev);
      const group = next.get(groupKey);
      if (!group) return prev;
      next.set(groupKey, { ...group, activeTabId: id });
      return next;
    });
  }, [groupKey]);

  const renameTab = useCallback((id: string, label: string) => {
    setGroups((prev) => {
      const next = new Map(prev);
      const group = next.get(groupKey);
      if (!group) return prev;
      next.set(groupKey, { ...group, tabs: group.tabs.map((t) => (t.id === id ? { ...t, label } : t)) });
      return next;
    });
  }, [groupKey]);

  const reorderTabs = useCallback((fromIndex: number, toIndex: number) => {
    setGroups((prev) => {
      const next = new Map(prev);
      const group = next.get(groupKey);
      if (!group) return prev;
      const tabs = [...group.tabs];
      const [moved] = tabs.splice(fromIndex, 1);
      tabs.splice(toIndex, 0, moved);
      next.set(groupKey, { ...group, tabs });
      return next;
    });
  }, [groupKey]);

  const togglePanel = useCallback(() => {
    setGlobal((prev) => {
      if (prev.isOpen) return { ...prev, isOpen: false };
      if (currentGroup.tabs.length === 0) return prev;
      return { ...prev, isOpen: true };
    });
  }, [currentGroup.tabs.length]);

  const setPtyId = useCallback((tabId: string, ptyId: number, ptyToken?: string) => {
    setGroups((prev) => {
      const next = new Map(prev);
      for (const [key, group] of next) {
        if (group.tabs.some((t) => t.id === tabId)) {
          next.set(key, { ...group, tabs: group.tabs.map((t) => (t.id === tabId ? { ...t, ptyId, ptyToken: ptyToken ?? null } : t)) });
          break;
        }
      }
      return next;
    });
  }, []);

  const resizePanel = useCallback((height: number) => {
    const clamped = Math.max(MIN_HEIGHT, Math.min(height, maxHeightRef.current));
    setGlobal((prev) => ({ ...prev, panelHeight: clamped }));
  }, []);

  const setMaxHeight = useCallback((maxH: number) => { maxHeightRef.current = maxH; }, []);

  const removeGroup = useCallback((key: string) => {
    setGroups((prev) => { const next = new Map(prev); next.delete(key); return next; });
  }, []);

  const getGroupPtyIds = useCallback((key: string): number[] => {
    const group = groups.get(key);
    if (!group) return [];
    return group.tabs.filter((t) => t.ptyId !== null).map((t) => t.ptyId!);
  }, [groups]);

  const getGroupPtyEntries = useCallback((key: string): { id: number; token: string }[] => {
    const group = groups.get(key);
    if (!group) return [];
    return group.tabs
      .filter((t) => t.ptyId !== null && t.ptyToken !== null)
      .map((t) => ({ id: t.ptyId!, token: t.ptyToken! }));
  }, [groups]);

  return {
    tabs: currentGroup.tabs,
    activeTabId: currentGroup.activeTabId,
    isOpen: global.isOpen,
    panelHeight: global.panelHeight,
    allTabs,
    addTab,
    closeTab,
    setActiveTab,
    renameTab,
    reorderTabs,
    togglePanel,
    setPtyId,
    resizePanel,
    setMaxHeight,
    removeGroup,
    getGroupPtyIds,
    getGroupPtyEntries,
    groupKey,
  };
}
