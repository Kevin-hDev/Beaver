import { useEffect } from "react";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";
import { matchesAppShortcut } from "@/lib/app-shortcuts";

interface AgentLocalShortcutsParams {
  activeSessionId?: string | null;
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
}

export function useAgentLocalShortcuts(params: AgentLocalShortcutsParams) {
  const surfaceActive = useAppSurfaceActive();
  const {
    activeSessionId,
    onToggleTerminal,
    onTogglePreview,
  } = params;

  useEffect(() => {
    if (!surfaceActive) return;

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
    activeSessionId, onTogglePreview, onToggleTerminal, surfaceActive,
  ]);
}

function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement
    || target instanceof HTMLTextAreaElement
    || (target instanceof HTMLElement && target.isContentEditable);
}
