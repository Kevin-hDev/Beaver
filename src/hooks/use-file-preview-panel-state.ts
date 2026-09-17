import { useCallback, useMemo, useState, type SetStateAction } from "react";
import {
  FILE_PREVIEW_DEFAULT_EXTRA_WIDTH,
  readStoredFilePreviewPanel,
  writeStoredFilePreviewPanel,
  type StoredFilePreviewPanel,
} from "./file-preview-storage";

interface PanelState extends StoredFilePreviewPanel {
  extraWidth: number;
}

interface KeyedPanelState {
  key: string;
  value: PanelState;
}

function stateKey(sessionId: string | null): string {
  return sessionId ?? "none";
}

function loadPanelState(sessionId: string | null): PanelState {
  return {
    ...readStoredFilePreviewPanel(sessionId),
    extraWidth: FILE_PREVIEW_DEFAULT_EXTRA_WIDTH,
  };
}

function applyAction<T>(current: T, action: SetStateAction<T>): T {
  return typeof action === "function" ? (action as (value: T) => T)(current) : action;
}

export function useFilePreviewPanelState(sessionId: string | null) {
  const key = stateKey(sessionId);
  const loadedState = useMemo(() => loadPanelState(sessionId), [sessionId]);
  const [stored, setStored] = useState<KeyedPanelState>(() => ({
    key,
    value: loadedState,
  }));
  const state = stored.key === key ? stored.value : loadedState;

  const updatePanel = useCallback((updater: (current: PanelState) => PanelState) => {
    setStored((currentStored) => {
      const current = currentStored.key === key
        ? currentStored.value
        : loadPanelState(sessionId);
      const next = updater(current);
      if (
        next.width === current.width &&
        next.extraWidth === current.extraWidth
      ) return currentStored;
      writeStoredFilePreviewPanel(sessionId, {
        width: next.width,
      });
      return { key, value: next };
    });
  }, [key, sessionId]);

  const setWidth = useCallback((action: SetStateAction<number>) => {
    updatePanel((current) => ({ ...current, width: applyAction(current.width, action) }));
  }, [updatePanel]);

  const setExtraWidth = useCallback((action: SetStateAction<number>) => {
    updatePanel((current) => ({ ...current, extraWidth: applyAction(current.extraWidth, action) }));
  }, [updatePanel]);

  return {
    ...state,
    setWidth,
    setExtraWidth,
  };
}
