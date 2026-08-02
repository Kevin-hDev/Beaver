import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { useAppNavigationActions } from "./use-app-navigation-actions";
import i18n from "@/i18n";

const MAX_ALLOWED_PATHS = 32;
const MAX_PATH_CHARS = 4_096;

interface DirectoryAccessDecision {
  allowed: boolean;
  allowed_paths: string[];
}

export interface BlockedDirectoryAccess {
  allowedPaths: string[];
}

function parseDecision(value: unknown): DirectoryAccessDecision {
  if (!value || typeof value !== "object") throw new Error("Invalid access decision");
  const decision = value as Record<string, unknown>;
  if (typeof decision.allowed !== "boolean" || !Array.isArray(decision.allowed_paths)) {
    throw new Error("Invalid access decision");
  }
  const allowedPaths = decision.allowed_paths;
  if (
    allowedPaths.length < 1
    || allowedPaths.length > MAX_ALLOWED_PATHS
    || allowedPaths.some((path) => typeof path !== "string" || path.length < 1 || path.length > MAX_PATH_CHARS)
  ) {
    throw new Error("Invalid access decision");
  }
  return { allowed: decision.allowed, allowed_paths: allowedPaths as string[] };
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
      decision = parseDecision(raw);
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
        const retry = parseDecision(
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

  return { blocked, request, cancel, openSettings };
}
