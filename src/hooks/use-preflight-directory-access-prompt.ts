import { useCallback } from "react";
import type { DirectoryAccessPromptProps } from "@/components/agent-local/directory-access-prompt";
import { useAppNavigationActions } from "./use-app-navigation-actions";

export function usePreflightDirectoryAccessPrompt(
  allowedPaths: string[] | null,
  dismiss: () => void,
): DirectoryAccessPromptProps | undefined {
  const { openFileAccessSettings } = useAppNavigationActions();
  const openSettings = useCallback(() => {
    dismiss();
    openFileAccessSettings();
  }, [dismiss, openFileAccessSettings]);

  if (!allowedPaths) return undefined;
  return {
    allowedPaths,
    onCancel: dismiss,
    onSettings: openSettings,
  };
}
