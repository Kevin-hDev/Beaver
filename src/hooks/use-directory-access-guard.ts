import { useCallback, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { useAppNavigationActions } from "./use-app-navigation-actions";
import i18n from "@/i18n";
import {
  MAX_PATH_CHARS,
  parseDirectoryAccessDecision,
  type DirectoryAccessDecision,
} from "./directory-access-decision";

export interface BlockedDirectoryAccess {
  allowedPaths: string[];
}

export function useDirectoryAccessGuard() {
  const { openFileAccessSettings } = useAppNavigationActions();
  const [blocked, setBlocked] = useState<BlockedDirectoryAccess | null>(null);

  const request = useCallback(async (
    path: string,
    onAllowed: () => void | Promise<void>,
  ) => {
    if (!path || path.length > MAX_PATH_CHARS) {
      showToast(i18n.t("directoryAccess.error"), "error");
      return;
    }
    let decision: DirectoryAccessDecision;
    try {
      const raw = await invoke<unknown>("validate_session_directory_access", { path });
      decision = parseDirectoryAccessDecision(raw);
    } catch {
      showToast(i18n.t("directoryAccess.error"), "error");
      return;
    }
    if (!decision.allowed) {
      setBlocked({ allowedPaths: decision.allowed_paths });
      return;
    }
    setBlocked(null);
    try {
      await onAllowed();
    } catch {
      try {
        const retry = parseDirectoryAccessDecision(
          await invoke<unknown>("validate_session_directory_access", { path }),
        );
        if (!retry.allowed) {
          setBlocked({ allowedPaths: retry.allowed_paths });
          return;
        }
      } catch {
        // L’erreur visible reste générique et ne révèle aucun détail du backend.
      }
      showToast(i18n.t("errors.operationFailed"), "error");
    }
  }, []);

  const cancel = useCallback(() => setBlocked(null), []);
  const openSettings = useCallback(() => {
    setBlocked(null);
    openFileAccessSettings();
  }, [openFileAccessSettings]);

  const prompt = useMemo(() => blocked ? {
    allowedPaths: blocked.allowedPaths,
    onCancel: cancel,
    onSettings: openSettings,
  } : undefined, [blocked, cancel, openSettings]);

  return { prompt, request };
}
