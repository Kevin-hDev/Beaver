import { useCallback, useMemo } from "react";
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
    onNavChange?.({ terminalActiveTabId: id });
  }, [onNavChange, terminalState]);

  const addTab = useCallback((cwd?: string) => {
    const id = terminalState.addTab(cwd);
    onNavChange?.({ terminalOpen: true, terminalActiveTabId: id });
    return id;
  }, [onNavChange, terminalState]);

  const togglePanel = useCallback(() => {
    const nextOpen = !navState.terminalOpen;
    if (nextOpen && terminalState.tabs.length === 0) {
      const id = terminalState.addTab(terminalCwd);
      onNavChange?.({ terminalOpen: true, terminalActiveTabId: id });
      return;
    }
    terminalState.togglePanel();
    onNavChange?.({ terminalOpen: nextOpen });
  }, [navState.terminalOpen, onNavChange, terminalCwd, terminalState]);

  return useMemo(() => ({
    ...terminalState,
    isOpen: navState.terminalOpen,
    activeTabId: navState.terminalActiveTabId,
    addTab,
    setActiveTab,
    togglePanel,
  }), [
    addTab, navState.terminalActiveTabId, navState.terminalOpen,
    setActiveTab, terminalState, togglePanel,
  ]);
}
