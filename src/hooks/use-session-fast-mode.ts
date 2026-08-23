import { useCallback, useReducer, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { admissionErrorMessage } from "@/lib/admission-error";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";

const MAX_PENDING_FAST_MODE_MUTATIONS = 32;

export function useSessionFastMode(refresh: () => Promise<void>) {
  const pendingIdsRef = useRef(new Set<string>());
  const [, refreshPendingState] = useReducer((value: number) => value + 1, 0);

  const setFastMode = useCallback(async (id: string, enabled: boolean) => {
    if (pendingIdsRef.current.has(id)) return;
    if (pendingIdsRef.current.size >= MAX_PENDING_FAST_MODE_MUTATIONS) return;

    pendingIdsRef.current.add(id);
    refreshPendingState();
    try {
      try {
        await invoke<boolean>("set_session_fast_mode", { id, enabled });
      } catch (error) {
        showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      }
      await refresh();
    } finally {
      pendingIdsRef.current.delete(id);
      refreshPendingState();
    }
  }, [refresh]);

  const isFastModePending = useCallback(
    (id: string) => pendingIdsRef.current.has(id),
    [],
  );

  return { setFastMode, isFastModePending };
}
