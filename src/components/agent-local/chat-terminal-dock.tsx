import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { useAgentLocalControlledTerminal } from "@/hooks/use-agent-local-controlled-terminal";

interface ChatTerminalDockProps {
  terminalState: ReturnType<typeof useAgentLocalControlledTerminal>;
}

export function ChatTerminalDock({ terminalState }: ChatTerminalDockProps) {
  return (
    <TerminalPanel
      tabs={terminalState.tabs}
      activeTabId={terminalState.activeTabId}
      allTabs={terminalState.allTabs()}
      activeGroupKey={terminalState.groupKey}
      isOpen={terminalState.isOpen}
      panelHeight={terminalState.panelHeight}
      onAddTab={terminalState.addTab}
      onCloseTab={terminalState.closeTab}
      onSelectTab={terminalState.setActiveTab}
      onRenameTab={terminalState.renameTab}
      onReorderTabs={terminalState.reorderTabs}
      onTogglePanel={terminalState.togglePanel}
      onPtyReady={terminalState.setPtyId}
      onTabActivity={terminalState.setTabActivity}
      onResize={terminalState.resizePanel}
      onSetMaxHeight={terminalState.setMaxHeight}
    />
  );
}
