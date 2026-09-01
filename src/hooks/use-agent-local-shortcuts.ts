import { useEffect } from "react";
import { matchesAppShortcut } from "@/lib/app-shortcuts";

interface AgentLocalShortcutsParams {
  activeSessionId?: string | null;
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
}

export function useAgentLocalShortcuts(params: AgentLocalShortcutsParams) {
  const {
    activeSessionId,
    onToggleTerminal,
    onTogglePreview,
  } = params;

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const toggleTerminal = matchesAppShortcut(event, "toggleTerminal");
      const togglePreview = matchesAppShortcut(event, "togglePreview");
      if (!activeSessionId || isEditableTarget(event.target)) return;
      if (togglePreview) {
        event.preventDefault();
        onTogglePreview();
        return;
      }
      if (!toggleTerminal) return;
      event.preventDefault();
      onToggleTerminal();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    activeSessionId, onTogglePreview, onToggleTerminal,
  ]);
}

function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement
    || target instanceof HTMLTextAreaElement
    || (target instanceof HTMLElement && target.isContentEditable);
}
