import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useFsEvent } from "@/hooks/use-fs-event";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import type {
  CompressionDeleteResult,
  CompressionProfileInput,
  CompressionProfilesView,
} from "@/types/compression-profile.generated";

interface QueuedSave {
  input: CompressionProfileInput;
  resolve: (saved: boolean) => void;
}

export interface CompressionProfilesController {
  view: CompressionProfilesView | null;
  busy: boolean;
  selectGlobal(profileId: string): Promise<boolean>;
  save(input: CompressionProfileInput): Promise<boolean>;
  create(sourceProfileId: string, name: string): Promise<boolean>;
  rename(profileId: string, name: string): Promise<boolean>;
  resetBeaver(): Promise<boolean>;
  deleteProfile(profileId: string): Promise<CompressionDeleteResult | null>;
  undoDelete(token: string): Promise<boolean>;
  refresh(): Promise<void>;
}

function ordered(view: CompressionProfilesView): CompressionProfilesView {
  return {
    ...view,
    profiles: [...view.profiles].sort((left, right) => {
      if (left.id === "beaver") return -1;
      if (right.id === "beaver") return 1;
      return 0;
    }),
  };
}

export function useCompressionProfiles(): CompressionProfilesController {
  const [view, setView] = useState<CompressionProfilesView | null>(null);
  const [busy, setBusy] = useState(false);
  const viewRef = useRef<CompressionProfilesView | null>(null);
  const mountedRef = useRef(true);
  const saveRunningRef = useRef(false);
  const queuedSaveRef = useRef<QueuedSave | null>(null);

  const applyView = useCallback((next: CompressionProfilesView) => {
    const normalized = ordered(next);
    viewRef.current = normalized;
    if (mountedRef.current) setView(normalized);
  }, []);

  const refresh = useCallback(async () => {
    try {
      applyView(await invoke<CompressionProfilesView>("get_compression_profiles"));
    } catch {
      if (!viewRef.current) showToast(i18n.t("errors.operationFailed"), "error");
    }
  }, [applyView]);

  useEffect(() => {
    mountedRef.current = true;
    void refresh();
    return () => {
      mountedRef.current = false;
      queuedSaveRef.current?.resolve(false);
      queuedSaveRef.current = null;
    };
  }, [refresh]);

  useFsEvent("fs:compression-profiles-changed", () => { void refresh(); });

  const selectGlobal = useCallback(async (profileId: string): Promise<boolean> => {
    if (busy || saveRunningRef.current) return false;
    setBusy(true);
    try {
      const next = await invoke<CompressionProfilesView>(
        "select_global_compression_profile",
        { profileId },
      );
      applyView(next);
      return true;
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
      return false;
    } finally {
      if (mountedRef.current) setBusy(false);
    }
  }, [applyView, busy]);

  const mutateView = useCallback(async (
    command: string,
    args?: Record<string, unknown>,
  ): Promise<boolean> => {
    if (busy || saveRunningRef.current) return false;
    setBusy(true);
    try {
      applyView(await invoke<CompressionProfilesView>(command, args));
      return true;
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
      return false;
    } finally {
      if (mountedRef.current) setBusy(false);
    }
  }, [applyView, busy]);

  const runSaveQueue = useCallback(async (first: QueuedSave) => {
    saveRunningRef.current = true;
    if (mountedRef.current) setBusy(true);
    let current: QueuedSave | null = first;
    while (current) {
      const request = current;
      try {
        const next: CompressionProfilesView = await invoke("save_compression_profile", {
          input: request.input,
        });
        request.resolve(true);
        const pending = queuedSaveRef.current;
        queuedSaveRef.current = null;
        if (!pending) {
          applyView(next);
          current = null;
        } else {
          const revision = next.profiles.find((item) => item.id === pending.input.id)?.revision;
          current = {
            ...pending,
            input: revision == null ? pending.input : { ...pending.input, revision },
          };
        }
      } catch {
        request.resolve(false);
        queuedSaveRef.current?.resolve(false);
        queuedSaveRef.current = null;
        showToast(i18n.t("errors.saveFailed"), "error");
        await refresh();
        current = null;
      }
    }
    saveRunningRef.current = false;
    if (mountedRef.current) setBusy(false);
  }, [applyView, refresh]);

  const save = useCallback((input: CompressionProfileInput): Promise<boolean> => (
    new Promise((resolve) => {
      const queued = { input, resolve };
      if (saveRunningRef.current) {
        queuedSaveRef.current?.resolve(false);
        queuedSaveRef.current = queued;
        return;
      }
      void runSaveQueue(queued);
    })
  ), [runSaveQueue]);

  const deleteProfile = useCallback(async (
    profileId: string,
  ): Promise<CompressionDeleteResult | null> => {
    if (busy || saveRunningRef.current) return null;
    setBusy(true);
    try {
      const result = await invoke<CompressionDeleteResult>("delete_compression_profile", {
        profileId,
      });
      applyView(result.view);
      return result;
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
      return null;
    } finally {
      if (mountedRef.current) setBusy(false);
    }
  }, [applyView, busy]);

  return {
    view,
    busy,
    selectGlobal,
    save,
    create: (sourceProfileId, name) => mutateView("create_compression_profile", {
      sourceProfileId,
      name,
    }),
    rename: (profileId, name) => mutateView("rename_compression_profile", { profileId, name }),
    resetBeaver: () => mutateView("reset_beaver_compression_profile"),
    deleteProfile,
    undoDelete: (undoToken) => mutateView("undo_delete_compression_profile", { undoToken }),
    refresh,
  };
}
