import { useEffect } from "react";

interface AgentLocalShortcutsParams {
  activeSessionId?: string | null;
  terminalOpen: boolean;
  terminalTabsCount: number;
  terminalCwd: string;
  onAddTerminalTab: (cwd: string) => void;
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
}

export function useAgentLocalShortcuts(params: AgentLocalShortcutsParams) {
  const {
    activeSessionId,
    terminalOpen,
    terminalTabsCount,
    terminalCwd,
    onAddTerminalTab,
    onToggleTerminal,
    onTogglePreview,
  } = params;

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const onMac = navigator.userAgent.includes("Mac");
      const mod = onMac ? event.metaKey : event.ctrlKey;
      const toggleTerminal = mod && !event.altKey && event.code === "KeyJ";
      const togglePreview = mod && event.altKey && event.code === "KeyB";
      if (!activeSessionId || isEditableTarget(event.target)) return;
      if (togglePreview) {
        event.preventDefault();
        onTogglePreview();
        return;
      }
      if (!toggleTerminal) return;
      event.preventDefault();
      if (!terminalOpen && terminalTabsCount === 0) {
        onAddTerminalTab(terminalCwd);
      } else {
        onToggleTerminal();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    activeSessionId, onAddTerminalTab, onTogglePreview, onToggleTerminal,
    terminalCwd, terminalOpen, terminalTabsCount,
  ]);
}

function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement
    || target instanceof HTMLTextAreaElement
    || (target instanceof HTMLElement && target.isContentEditable);
}
