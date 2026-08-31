import { useCallback, useEffect, useMemo, useRef } from "react";
import type { useTerminal } from "@/hooks/use-terminal";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

interface Args {
  navState: AgentLocalNavState;
  terminalState: ReturnType<typeof useTerminal>;
  terminalCwd: string;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalControlledTerminal({ navState, terminalState, terminalCwd, onNavChange }: Args) {
  const setActiveTab = useCallback((id: string) => {
    terminalState.setActiveTab(id);
  }, [terminalState]);

  const addTab = useCallback((cwd?: string) => {
    const id = terminalState.addTab(cwd);
    if (id !== null) onNavChange?.({ terminalOpen: true });
    return id;
  }, [onNavChange, terminalState]);

  const closeTab = useCallback((id: string): void => {
    terminalState.closeTab(id);
  }, [terminalState]);

  const closeTabInGroup = useCallback((groupKey: string, id: string): void => {
    terminalState.closeTabInGroup(groupKey, id);
  }, [terminalState]);

  const togglePanel = useCallback(() => {
    const nextOpen = !navState.terminalOpen;
    if (nextOpen && terminalState.tabs.length === 0) {
      addTab(terminalCwd);
      return;
    }
    onNavChange?.({ terminalOpen: nextOpen });
  }, [addTab, navState.terminalOpen, onNavChange, terminalCwd, terminalState.tabs.length]);

  const previousGroup = useRef({
    groupKey: terminalState.groupKey,
    count: terminalState.tabs.length,
  });
  useEffect(() => {
    const previous = previousGroup.current;
    const current = { groupKey: terminalState.groupKey, count: terminalState.tabs.length };
    previousGroup.current = current;
    if (navState.terminalOpen && previous.groupKey === current.groupKey
      && previous.count > 0 && current.count === 0) {
      onNavChange?.({ terminalOpen: false });
    }
  }, [navState.terminalOpen, onNavChange, terminalState.groupKey, terminalState.tabs.length]);

  return useMemo(() => ({
    ...terminalState,
    isOpen: navState.terminalOpen,
    activeTabId: terminalState.activeTabId,
    addTab,
    closeTab,
    closeTabInGroup,
    setActiveTab,
    togglePanel,
  }), [
    addTab, closeTab, closeTabInGroup, navState.terminalOpen,
    setActiveTab, terminalState, togglePanel,
  ]);
}
