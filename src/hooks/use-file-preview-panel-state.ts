import { useCallback, useState, type SetStateAction } from "react";
import {
  FILE_PREVIEW_DEFAULT_EXTRA_WIDTH,
  readStoredFilePreviewPanel,
  writeStoredFilePreviewPanel,
  type StoredFilePreviewPanel,
} from "./file-preview-storage";

interface PanelState extends StoredFilePreviewPanel {
  open: boolean;
  fullscreen: boolean;
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
    open: false,
    fullscreen: false,
    extraWidth: FILE_PREVIEW_DEFAULT_EXTRA_WIDTH,
  };
}

function applyAction<T>(current: T, action: SetStateAction<T>): T {
  return typeof action === "function" ? (action as (value: T) => T)(current) : action;
}

export function useFilePreviewPanelState(sessionId: string | null) {
  const key = stateKey(sessionId);
  const [stored, setStored] = useState<KeyedPanelState>(() => ({
    key,
    value: loadPanelState(sessionId),
  }));
  const state = stored.key === key ? stored.value : loadPanelState(sessionId);

  const updatePanel = useCallback((updater: (current: PanelState) => PanelState) => {
    setStored((currentStored) => {
      const current = currentStored.key === key
        ? currentStored.value
        : loadPanelState(sessionId);
      const next = updater(current);
      if (
        next.open === current.open &&
        next.fullscreen === current.fullscreen &&
        next.width === current.width &&
        next.extraWidth === current.extraWidth
      ) return currentStored;
      writeStoredFilePreviewPanel(sessionId, {
        width: next.width,
      });
      return { key, value: next };
    });
  }, [key, sessionId]);

  const setOpen = useCallback((action: SetStateAction<boolean>) => {
    updatePanel((current) => ({ ...current, open: applyAction(current.open, action) }));
  }, [updatePanel]);

  const setFullscreen = useCallback((action: SetStateAction<boolean>) => {
    updatePanel((current) => ({ ...current, fullscreen: applyAction(current.fullscreen, action) }));
  }, [updatePanel]);

  const setWidth = useCallback((action: SetStateAction<number>) => {
    updatePanel((current) => ({ ...current, width: applyAction(current.width, action) }));
  }, [updatePanel]);

  const setExtraWidth = useCallback((action: SetStateAction<number>) => {
    updatePanel((current) => ({ ...current, extraWidth: applyAction(current.extraWidth, action) }));
  }, [updatePanel]);

  return {
    ...state,
    setOpen,
    setFullscreen,
    setWidth,
    setExtraWidth,
  };
}
