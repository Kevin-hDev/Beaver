import { useCallback, type Dispatch, type SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import { extensionErrorKey } from "@/lib/extension-errors";
import type { ExtensionInstallOutcome } from "@/lib/extension-install";
import { parseExtensionRecord } from "@/lib/extension-records";

export function useExtensionInstall(
  refresh: () => Promise<void>,
  setOperationError: Dispatch<SetStateAction<string | null>>,
) {
  return useCallback(async (
    command: string,
    payload: Record<string, unknown>,
  ): Promise<ExtensionInstallOutcome> => {
    setOperationError(null);
    try {
      const record = parseExtensionRecord(await invoke<unknown>(command, payload));
      await refresh();
      return { record, errorKey: null };
    } catch (error) {
      const errorKey = extensionErrorKey(error, "extensions.errors.operation");
      setOperationError(errorKey);
      return { record: null, errorKey };
    }
  }, [refresh, setOperationError]);
}
