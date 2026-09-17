import { useCallback, useEffect, useMemo, useState, type SetStateAction } from "react";
import { fileNameFromPath, normalizeFileOperationPath } from "@/lib/file-preview-utils";
import type { AgentPlanRun } from "@/types/agent";
import type { FileOperation, FilePreviewActiveTab, FilePreviewListMode } from "@/types/file-preview";
import type { AgentLocalWorkspaceState } from "@/types/navigation";
import {
  readStoredFilePreviewTabs,
  writeStoredFilePreviewTabs,
} from "./file-preview-storage";
import { useFilePreviewPanelState } from "./use-file-preview-panel-state";
import { useFilePreviewResize } from "./use-file-preview-resize";
import { usePreviewFallbackExistence } from "./use-preview-fallback-existence";
import { usePrunePreviewTabs } from "./use-prune-preview-tabs";
const MAX_TABS = 6;
const MAX_TRACKED_FALLBACK_SESSIONS = 32;
const EMPTY_FALLBACKS: FileOperation[] = [];

interface FilePreviewViewState {
  open: boolean;
  fullscreen: boolean;
  activeTab: FilePreviewActiveTab;
  onChange?: (partial: Partial<AgentLocalWorkspaceState>) => void;
}

export function useFilePreview(
  sessionId: string | null,
  operations: FileOperation[],
  baseDir: string | undefined,
  view: FilePreviewViewState,
) {
  const { open, fullscreen, activeTab, onChange } = view;
  const {
    width,
    extraWidth,
    setWidth,
    setExtraWidth,
  } = useFilePreviewPanelState(sessionId);
  const [listMode, setListMode] = useState<FilePreviewListMode>("latest");
  const [tabIds, setTabIds] = useState<string[]>(() => readStoredFilePreviewTabs(sessionId));
  const [fallbacksBySession, setFallbacksBySession] = useState<Map<string | null, FileOperation[]>>(
    () => new Map(),
  );
  const fallbackOps = fallbacksBySession.get(sessionId) ?? EMPTY_FALLBACKS;
  const setFallbackOps = useCallback((action: SetStateAction<FileOperation[]>) => {
    setFallbacksBySession((stored) => {
      const current = stored.get(sessionId) ?? EMPTY_FALLBACKS;
      const next = typeof action === "function" ? action(current) : action;
      if (next === current) return stored;

      const updated = new Map(stored);
      updated.delete(sessionId);
      if (next.length > 0) updated.set(sessionId, next);
      while (updated.size > MAX_TRACKED_FALLBACK_SESSIONS) {
        const oldest = updated.keys().next();
        if (oldest.done) break;
        updated.delete(oldest.value);
      }
      return updated;
    });
  }, [sessionId]);
  const { resizing, startResize } = useFilePreviewResize({
    width,
    extraWidth,
    setWidth,
  });

  const allOperations = useMemo(() => [...fallbackOps, ...operations], [operations, fallbackOps]);
  const filesystemFallbacks = useMemo(
    () => fallbackOps.filter((operation) => !operation.source && !operation.recordedStatus),
    [fallbackOps],
  );
  const operationById = useMemo(() => new Map(allOperations.map((op) => [op.id, op])), [allOperations]);
  const tabs = tabIds.flatMap((id) => operationById.get(id) ?? []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset on session change is intentional
    setTabIds(readStoredFilePreviewTabs(sessionId));
    setListMode("latest");
  }, [sessionId]);

  const setOpen = useCallback((action: SetStateAction<boolean>) => {
    const next = typeof action === "function" ? action(open) : action;
    if (next !== open) onChange?.({ previewOpen: next });
  }, [onChange, open]);
  const setFullscreen = useCallback((action: SetStateAction<boolean>) => {
    const next = typeof action === "function" ? action(fullscreen) : action;
    if (next !== fullscreen) onChange?.({ previewFullscreen: next });
  }, [fullscreen, onChange]);
  const setActiveTab = useCallback((action: SetStateAction<FilePreviewActiveTab>) => {
    const next = typeof action === "function" ? action(activeTab) : action;
    if (next !== activeTab) onChange?.({ previewActiveTab: next });
  }, [activeTab, onChange]);

  usePrunePreviewTabs(operationById, setTabIds, setActiveTab);

  useEffect(() => {
    writeStoredFilePreviewTabs(sessionId, tabIds);
  }, [sessionId, tabIds]);

  const removeMissingFallbacks = useCallback((missingKeys: Set<string>) => {
    setFallbackOps((items) => items.filter((item) => (
      !missingKeys.has(normalizeFileOperationPath(item.path))
    )));
  }, [setFallbackOps]);
  usePreviewFallbackExistence(filesystemFallbacks, baseDir, removeMissingFallbacks);

  const openOperation = useCallback((operation: FileOperation) => {
    setFallbackOps((items) => [operation, ...items.filter((item) => item.id !== operation.id)].slice(0, MAX_TABS));
    onChange?.({ previewOpen: true, previewActiveTab: operation.id });
    setTabIds((ids) => {
      const next = [operation.id, ...ids.filter((id) => id !== operation.id)];
      return next.slice(0, MAX_TABS);
    });
    return operation.id;
  }, [onChange, setFallbackOps]);

  const openFullPath = useCallback((path: string) => {
    const fallback: FileOperation = {
      id: `read:${path}`,
      path,
      name: fileNameFromPath(path),
      type: "read",
      timestamp: new Date().toISOString(),
      additions: 0,
      deletions: 0,
    };
    setFallbackOps((items) => [fallback, ...items.filter((item) => item.id !== fallback.id)].slice(0, MAX_TABS));
    return openOperation(fallback);
  }, [openOperation, setFallbackOps]);

  const openPath = useCallback((path: string) => {
    const operation = [...operations].reverse().find((op) => op.path === path);
    if (operation) {
      return openOperation(operation);
    }
    return openFullPath(path);
  }, [operations, openOperation, openFullPath]);

  const openPlan = useCallback((plan: AgentPlanRun) => {
    const operation: FileOperation = {
      id: `plan:${plan.id}`,
      path: plan.path,
      name: plan.title,
      type: "read",
      kind: "plan",
      timestamp: plan.updated_at,
      additions: 0,
      deletions: 0,
    };
    setFallbackOps((items) => [operation, ...items.filter((item) => item.id !== operation.id)].slice(0, MAX_TABS));
    return openOperation(operation);
  }, [openOperation, setFallbackOps]);

  const closeTab = useCallback((id: string) => {
    setTabIds((ids) => ids.filter((tabId) => tabId !== id));
    if (activeTab === id) onChange?.({ previewActiveTab: "summary" });
  }, [activeTab, onChange]);

  const closePanel = useCallback(() => {
    onChange?.({ previewOpen: false, previewFullscreen: false });
    setExtraWidth(0);
  }, [onChange, setExtraWidth]);

  const toggleOpen = useCallback(() => {
    onChange?.({
      previewOpen: !open,
      previewFullscreen: open ? false : fullscreen,
      previewActiveTab: activeTab || "summary",
    });
  }, [activeTab, fullscreen, onChange, open]);

  return {
    open,
    fullscreen,
    activeTab,
    listMode,
    tabs,
    width,
    extraWidth,
    resizing,
    setOpen,
    setFullscreen,
    setExtraWidth,
    setActiveTab,
    setListMode,
    toggleOpen,
    closePanel,
    openOperation,
    openPath,
    openFullPath,
    openPlan,
    closeTab,
    startResize,
  };
}
