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

interface PreviewSyncOpts {
  navState: AgentLocalNavState;
  filePreview: FilePreviewPanelState;
}

export function useAgentLocalPreviewSync({ navState, filePreview }: PreviewSyncOpts) {
  const restoredPreviewNavKey = useRef<string | null>(null);
  const restoredSessionId = useRef<string | null>(null);
  const previewNavKey = JSON.stringify([
    navState.sessionId,
    navState.previewOpen,
    navState.previewActiveTab,
    navState.previewFullscreen,
  ]);

  useEffect(() => {
    if (restoredPreviewNavKey.current === previewNavKey) return;
    const sessionChanged = restoredSessionId.current !== navState.sessionId;
    restoredPreviewNavKey.current = previewNavKey;
    restoredSessionId.current = navState.sessionId;
    if (filePreview.open !== navState.previewOpen) {
      filePreview.setOpen(navState.previewOpen);
    }
    // Le reset interne de useFilePreview s'exécute dans le même cycle. Lors
    // d'une bascule, republier même une valeur apparemment identique garantit
    // que la restauration de la nouvelle session gagne toujours.
    if (sessionChanged || filePreview.activeTab !== navState.previewActiveTab) {
      filePreview.setActiveTab(navState.previewActiveTab);
    }
    if (filePreview.fullscreen !== navState.previewFullscreen) {
      filePreview.setFullscreen(navState.previewFullscreen);
    }
  }, [filePreview, navState.previewActiveTab, navState.previewFullscreen,
    navState.previewOpen, navState.sessionId, previewNavKey]);
}
