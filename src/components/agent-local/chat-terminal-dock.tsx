import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { useTerminal } from "@/hooks/use-terminal";

interface ChatTerminalDockProps {
  terminalState: ReturnType<typeof useTerminal>;
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
      onResize={terminalState.resizePanel}
      onSetMaxHeight={terminalState.setMaxHeight}
    />
  );
}
