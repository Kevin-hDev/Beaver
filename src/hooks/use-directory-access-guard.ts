import { useCallback, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { localStoreErrorMessage } from "@/lib/local-store-error";
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
    onAllowed: () => boolean | void | Promise<boolean | void>,
  ): Promise<boolean> => {
    if (!path || path.length > MAX_PATH_CHARS) {
      showToast(i18n.t("directoryAccess.error"), "error");
      return false;
    }
    let decision: DirectoryAccessDecision;
    try {
      const raw = await invoke<unknown>("validate_session_directory_access", { path });
      decision = parseDirectoryAccessDecision(raw);
    } catch (validationError) {
      void validationError;
      showToast(i18n.t("directoryAccess.error"), "error");
      return false;
    }
    if (!decision.allowed) {
      setBlocked({ allowedPaths: decision.allowed_paths });
      return false;
    }
    setBlocked(null);
    try {
      const accepted = await onAllowed();
      return accepted !== false;
    } catch (error) {
      try {
        const retry = parseDirectoryAccessDecision(
          await invoke<unknown>("validate_session_directory_access", { path }),
        );
        if (!retry.allowed) {
          setBlocked({ allowedPaths: retry.allowed_paths });
          return false;
        }
      } catch (retryError) {
        void retryError;
        // L’erreur visible reste générique et ne révèle aucun détail du backend.
      }
      showToast(localStoreErrorMessage(error, i18n.t), "error");
      return false;
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
