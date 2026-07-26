import { useCallback, useMemo, type SetStateAction } from "react";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { AgentPlanRun } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

function applyAction<T>(current: T, action: SetStateAction<T>): T {
  return typeof action === "function" ? (action as (value: T) => T)(current) : action;
}

interface Args {
  navState: AgentLocalNavState;
  filePreviewState: ReturnType<typeof useFilePreview>;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalControlledPreview({ navState, filePreviewState, onNavChange }: Args) {
  const publishPreviewTab = useCallback((tabId: string) => {
    onNavChange?.({
      previewOpen: true,
      previewActiveTab: tabId,
      panelMode: "preview",
    });
  }, [onNavChange]);

  const setOpen = useCallback((action: SetStateAction<boolean>) => {
    const next = applyAction(navState.previewOpen, action);
    filePreviewState.setOpen(next);
    onNavChange?.({ previewOpen: next });
  }, [filePreviewState, navState.previewOpen, onNavChange]);

  const setFullscreen = useCallback((action: SetStateAction<boolean>) => {
    const next = applyAction(navState.previewFullscreen, action);
    filePreviewState.setFullscreen(next);
    onNavChange?.({ previewFullscreen: next });
  }, [filePreviewState, navState.previewFullscreen, onNavChange]);

  const setActiveTab = useCallback((action: SetStateAction<string>) => {
    const next = applyAction(navState.previewActiveTab, action);
    filePreviewState.setActiveTab(next);
    onNavChange?.({ previewActiveTab: next });
  }, [filePreviewState, navState.previewActiveTab, onNavChange]);

  const toggleOpen = useCallback(() => {
    const nextOpen = !navState.previewOpen;
    filePreviewState.setOpen(nextOpen);
    if (!nextOpen) filePreviewState.setFullscreen(false);
    onNavChange?.({
      previewOpen: nextOpen,
      previewFullscreen: nextOpen ? navState.previewFullscreen : false,
      previewActiveTab: navState.previewActiveTab || "summary",
      ...(nextOpen ? {} : { fileTreeOpen: false }),
    });
  }, [filePreviewState, navState.previewActiveTab, navState.previewFullscreen, navState.previewOpen, onNavChange]);

  const closePanel = useCallback(() => {
    filePreviewState.closePanel();
    onNavChange?.({ previewOpen: false, previewFullscreen: false, fileTreeOpen: false });
  }, [filePreviewState, onNavChange]);

  const openOperation = useCallback((operation: FileOperation) => {
    const tabId = filePreviewState.openOperation(operation);
    publishPreviewTab(tabId);
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const openPath = useCallback((path: string) => {
    const tabId = filePreviewState.openPath(path);
    publishPreviewTab(tabId);
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const openFullPath = useCallback((path: string) => {
    const tabId = filePreviewState.openFullPath(path);
    publishPreviewTab(tabId);
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const openPlan = useCallback((plan: AgentPlanRun) => {
    const tabId = filePreviewState.openPlan(plan);
    publishPreviewTab(tabId);
    return tabId;
  }, [filePreviewState, publishPreviewTab]);

  const closeTab = useCallback((id: string) => {
    filePreviewState.closeTab(id);
    if (navState.previewActiveTab === id) onNavChange?.({ previewActiveTab: "summary" });
  }, [filePreviewState, navState.previewActiveTab, onNavChange]);

  return useMemo(() => ({
    ...filePreviewState,
    open: navState.previewOpen,
    fullscreen: navState.previewFullscreen,
    activeTab: navState.previewActiveTab,
    setOpen,
    setFullscreen,
    setActiveTab,
    toggleOpen,
    closePanel,
    openOperation,
    openPath,
    openFullPath,
    openPlan,
    closeTab,
  }), [
    closePanel, closeTab, filePreviewState, navState.previewActiveTab,
    navState.previewFullscreen, navState.previewOpen, openOperation,
    openPath, openFullPath, openPlan, setActiveTab, setFullscreen, setOpen, toggleOpen,
  ]);
}
