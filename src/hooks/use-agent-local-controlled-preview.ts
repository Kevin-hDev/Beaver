import { useCallback, useMemo } from "react";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { AgentPlanRun } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";
import type { AgentLocalNavState, AgentLocalWorkspaceState } from "@/types/navigation";

interface Args {
  navState: AgentLocalNavState;
  filePreviewState: ReturnType<typeof useFilePreview>;
  onNavChange?: (partial: Partial<AgentLocalWorkspaceState>) => void;
}

export function useAgentLocalControlledPreview({ navState, filePreviewState, onNavChange }: Args) {
  const publishPreviewTab = useCallback(() => {
    onNavChange?.({ panelMode: "preview" });
  }, [onNavChange]);

  const toggleOpen = useCallback(() => {
    const nextOpen = !navState.previewOpen;
    filePreviewState.toggleOpen();
    if (!nextOpen) onNavChange?.({ fileTreeOpen: false });
  }, [filePreviewState, navState.previewOpen, onNavChange]);

  const closePanel = useCallback(() => {
    filePreviewState.closePanel();
    onNavChange?.({ fileTreeOpen: false });
  }, [filePreviewState, onNavChange]);

  const openOperation = useCallback((operation: FileOperation) => {
    const tabId = filePreviewState.openOperation(operation);
    publishPreviewTab();
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const openPath = useCallback((path: string) => {
    const tabId = filePreviewState.openPath(path);
    publishPreviewTab();
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const openFullPath = useCallback((path: string) => {
    const tabId = filePreviewState.openFullPath(path);
    publishPreviewTab();
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const openPlan = useCallback((plan: AgentPlanRun) => {
    const tabId = filePreviewState.openPlan(plan);
    publishPreviewTab();
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const closeTab = useCallback((id: string) => {
    filePreviewState.closeTab(id);
  }, [filePreviewState]);

  return useMemo(() => ({
    ...filePreviewState,
    toggleOpen,
    closePanel,
    openOperation,
    openPath,
    openFullPath,
    openPlan,
    closeTab,
  }), [
    closePanel, closeTab, filePreviewState, openOperation,
    openPath, openFullPath, openPlan, toggleOpen,
  ]);
}
