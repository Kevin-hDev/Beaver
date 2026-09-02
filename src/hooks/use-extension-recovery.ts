import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { extensionErrorKey } from "@/lib/extension-errors";
import { parseExtensionRecoveryState } from "@/lib/extension-recovery-state";
import type { ExtensionRecoveryState } from "@/types/extensions";

export const EMPTY_EXTENSION_RECOVERY: ExtensionRecoveryState = {
  extensionId: null,
  stage: null,
  attempts: null,
  canRetry: false,
  markerInvalid: false,
  recoverySnapshotAvailable: false,
};

type Run = (command: string, payload?: Record<string, unknown>) => Promise<void>;
type RunHost = (operation: () => Promise<void>) => Promise<void>;

export function useExtensionRecovery(
  run: Run,
  runHost: RunHost,
  setOperationError: (value: string | null) => void,
) {
  const [recovery, setRecovery] = useState(EMPTY_EXTENSION_RECOVERY);
  const [recoveryBusy, setRecoveryBusy] = useState(false);
  const recoveryBusyRef = useRef(false);
  const refreshGeneration = useRef(0);
  const refreshRecovery = useCallback(async () => {
    const generation = ++refreshGeneration.current;
    try {
      const value = await invoke<unknown>("get_extension_recovery_state");
      if (generation !== refreshGeneration.current) return;
      setRecovery(parseExtensionRecoveryState(value));
    } catch (error) {
      if (generation !== refreshGeneration.current) return;
      setOperationError(extensionErrorKey(error, "extensions.errors.operation"));
    }
  }, [setOperationError]);
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- synchronise l'état de reprise possédé par Rust
    void refreshRecovery();
  }, [refreshRecovery]);
  const runRecovery = useCallback(async (
    command: string,
    payload: Record<string, unknown> = {},
    ownsHost = false,
  ) => {
    if (recoveryBusyRef.current) return;
    recoveryBusyRef.current = true;
    setRecoveryBusy(true);
    try {
      const operation = () => run(command, payload);
      if (ownsHost) await runHost(operation);
      else await operation();
      await refreshRecovery();
    } finally {
      recoveryBusyRef.current = false;
      setRecoveryBusy(false);
    }
  }, [refreshRecovery, run, runHost]);

  return {
    recovery,
    recoveryBusy,
    refreshRecovery,
    keepDisabled: (extensionId: string) =>
      runRecovery("keep_extension_disabled", { extensionId }),
    retryLoad: (extensionId: string) =>
      runRecovery("retry_extension_load", { extensionId }, true),
    discardLoadingMarker: () =>
      runRecovery("discard_extension_loading_marker"),
    restoreRecoverySnapshot: () =>
      runRecovery("restore_extension_recovery_snapshot", {}, true),
  };
}
