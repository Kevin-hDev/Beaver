import { useCallback, useEffect, useMemo, useState } from "react";
import { fileNameFromPath, normalizeFileOperationPath } from "@/lib/file-preview-utils";
import type { AgentPlanRun } from "@/types/agent";
import type { FileOperation, FilePreviewActiveTab, FilePreviewListMode } from "@/types/file-preview";
import {
  readStoredFilePreviewTabs,
  writeStoredFilePreviewTabs,
} from "./file-preview-storage";
import { useFilePreviewPanelState } from "./use-file-preview-panel-state";
import { useFilePreviewResize } from "./use-file-preview-resize";
import { usePreviewFallbackExistence } from "./use-preview-fallback-existence";
import { usePrunePreviewTabs } from "./use-prune-preview-tabs";
const MAX_TABS = 6;

export function useFilePreview(
  sessionId: string | null,
  operations: FileOperation[],
  baseDir?: string,
) {
  const {
    open,
    fullscreen,
    width,
    extraWidth,
    setOpen,
    setFullscreen,
    setWidth,
    setExtraWidth,
  } = useFilePreviewPanelState(sessionId);
  const [activeTab, setActiveTab] = useState<FilePreviewActiveTab>("summary");
  const [listMode, setListMode] = useState<FilePreviewListMode>("latest");
  const [tabIds, setTabIds] = useState<string[]>(() => readStoredFilePreviewTabs(sessionId));
  const [fallbackOps, setFallbackOps] = useState<FileOperation[]>([]);
  const { resizing, startResize } = useFilePreviewResize({
    width,
    extraWidth,
    setWidth,
  });

  const allOperations = useMemo(() => [...operations, ...fallbackOps], [operations, fallbackOps]);
  const filesystemFallbacks = useMemo(
    () => fallbackOps.filter((operation) => !operation.source && !operation.recordedStatus),
    [fallbackOps],
  );
  const operationById = useMemo(() => new Map(allOperations.map((op) => [op.id, op])), [allOperations]);
  const tabs = tabIds.flatMap((id) => operationById.get(id) ?? []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset on session change is intentional
    setFallbackOps([]);
    setTabIds(readStoredFilePreviewTabs(sessionId));
    setActiveTab("summary"); setListMode("latest");
  }, [sessionId]);

  usePrunePreviewTabs(operationById, setTabIds, setActiveTab);

  useEffect(() => {
    writeStoredFilePreviewTabs(sessionId, tabIds);
  }, [sessionId, tabIds]);

  const removeMissingFallbacks = useCallback((missingKeys: Set<string>) => {
    setFallbackOps((items) => items.filter((item) => (
      !missingKeys.has(normalizeFileOperationPath(item.path))
    )));
  }, []);
  usePreviewFallbackExistence(filesystemFallbacks, baseDir, removeMissingFallbacks);

  const openOperation = useCallback((operation: FileOperation) => {
    setOpen(true);
    setFallbackOps((items) => operations.some((item) => item.id === operation.id) || items.some((item) => item.id === operation.id) ? items : [operation, ...items].slice(0, MAX_TABS));
    setActiveTab(operation.id);
    setTabIds((ids) => {
      const next = [operation.id, ...ids.filter((id) => id !== operation.id)];
      return next.slice(0, MAX_TABS);
    });
    return operation.id;
  }, [operations, setOpen]);

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
  }, [openOperation]);

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
  }, [openOperation]);

  const closeTab = useCallback((id: string) => {
    setTabIds((ids) => ids.filter((tabId) => tabId !== id));
    setActiveTab((current) => current === id ? "summary" : current);
  }, []);

  const closePanel = useCallback(() => {
    setOpen(false);
    setFullscreen(false);
    setExtraWidth(0);
  }, [setOpen, setFullscreen, setExtraWidth]);

  const toggleOpen = useCallback(() => {
    if (open) setFullscreen(false);
    setOpen(!open);
    setActiveTab((current) => current || "summary");
  }, [open, setOpen, setFullscreen]);

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
