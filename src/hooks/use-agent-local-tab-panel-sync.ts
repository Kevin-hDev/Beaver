import { useEffect, useRef, type SetStateAction } from "react";
import type { FilePreviewActiveTab } from "@/types/file-preview";
import type { AgentLocalNavState } from "@/types/navigation";

interface FilePreviewPanelState {
  open: boolean;
  activeTab: FilePreviewActiveTab;
  fullscreen: boolean;
  setOpen: (value: SetStateAction<boolean>) => void;
  setActiveTab: (value: SetStateAction<FilePreviewActiveTab>) => void;
  setFullscreen: (value: SetStateAction<boolean>) => void;
}

interface TerminalPanelState {
  isOpen: boolean;
  activeTabId: string | null;
  togglePanel: () => void;
  setActiveTab: (id: string) => void;
}

interface PanelSyncOpts {
  navState: AgentLocalNavState;
  filePreview: FilePreviewPanelState;
  terminal: TerminalPanelState;
}

export function useAgentLocalTabPanelSync({
  navState,
  filePreview,
  terminal,
}: PanelSyncOpts) {
  const {
    open: actualPreviewOpen,
    activeTab: actualPreviewActiveTab,
    fullscreen: actualPreviewFullscreen,
    setOpen,
    setActiveTab,
    setFullscreen,
  } = filePreview;
  const { isOpen: actualTerminalOpen, activeTabId, togglePanel, setActiveTab: setTerminalActiveTab } = terminal;
  const restoredPanelNavKey = useRef<string | null>(null);
  const panelNavKey = JSON.stringify([
    navState.previewOpen,
    navState.previewActiveTab,
    navState.previewFullscreen,
    navState.terminalOpen,
    navState.terminalActiveTabId,
  ]);

  useEffect(() => {
    if (restoredPanelNavKey.current === panelNavKey) return;
    restoredPanelNavKey.current = panelNavKey;
    if (actualPreviewOpen !== navState.previewOpen) {
      setOpen(navState.previewOpen);
    }
    if (actualPreviewActiveTab !== navState.previewActiveTab) {
      setActiveTab(navState.previewActiveTab);
    }
    if (actualPreviewFullscreen !== navState.previewFullscreen) {
      setFullscreen(navState.previewFullscreen);
    }
    if (actualTerminalOpen !== navState.terminalOpen) {
      togglePanel();
    }
    if (navState.terminalActiveTabId && activeTabId !== navState.terminalActiveTabId) {
      setTerminalActiveTab(navState.terminalActiveTabId);
    }
  }, [
    panelNavKey,
    navState.previewOpen, navState.previewActiveTab, navState.previewFullscreen,
    navState.terminalOpen, navState.terminalActiveTabId,
    actualPreviewOpen, actualPreviewActiveTab, actualPreviewFullscreen,
    setOpen, setActiveTab, setFullscreen,
    actualTerminalOpen, activeTabId, togglePanel, setTerminalActiveTab,
  ]);
}
