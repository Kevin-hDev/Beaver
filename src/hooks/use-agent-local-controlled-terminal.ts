import { useCallback, useMemo } from "react";
import type { useTerminal } from "@/hooks/use-terminal";
import type { AgentLocalNavState, AgentLocalWorkspaceState } from "@/types/navigation";

interface Args {
  navState: AgentLocalNavState;
  terminalState: ReturnType<typeof useTerminal>;
  terminalCwd: string;
  onNavChange?: (partial: Partial<AgentLocalWorkspaceState>) => void;
}

export function useAgentLocalControlledTerminal({ navState, terminalState, terminalCwd, onNavChange }: Args) {
  const isOpen = navState.terminalOpen
    && terminalState.loaded
    && terminalState.tabs.length > 0;

  const addTab = useCallback((cwd?: string) => {
    const id = terminalState.addTab(cwd);
    if (id !== null) onNavChange?.({ terminalOpen: true });
    return id;
  }, [onNavChange, terminalState]);

  const closeTab = useCallback((id: string): void => {
    const closesLastTab = terminalState.tabs.length === 1
      && terminalState.tabs[0]?.id === id;
    terminalState.closeTab(id);
    if (closesLastTab && navState.terminalOpen) onNavChange?.({ terminalOpen: false });
  }, [navState.terminalOpen, onNavChange, terminalState]);

  const closeTabInGroup = useCallback((groupKey: string, id: string): void => {
    const closesCurrentLastTab = groupKey === terminalState.groupKey
      && terminalState.tabs.length === 1
      && terminalState.tabs[0]?.id === id;
    terminalState.closeTabInGroup(groupKey, id);
    if (closesCurrentLastTab && navState.terminalOpen) {
      onNavChange?.({ terminalOpen: false });
    }
  }, [navState.terminalOpen, onNavChange, terminalState]);

  const togglePanel = useCallback(() => {
    const nextOpen = !isOpen;
    if (nextOpen && terminalState.tabs.length === 0) {
      addTab(terminalCwd);
      return;
    }
    onNavChange?.({ terminalOpen: nextOpen });
  }, [addTab, isOpen, onNavChange, terminalCwd, terminalState.tabs.length]);

  return useMemo(() => ({
    ...terminalState,
    isOpen,
    addTab,
    closeTab,
    closeTabInGroup,
    togglePanel,
  }), [
    addTab, closeTab, closeTabInGroup, isOpen, terminalState, togglePanel,
  ]);
}
