import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AGENT_SESSIONS_CHANGED, notifyAgentSessionsChanged } from "./agent-session-events";
import { useFsEvent } from "./use-fs-event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import type {
  CompressionProfileView,
  CompressionProfilesView,
  ResolvedCompressionProfileView,
} from "@/types/compression-profile.generated";

export interface SessionCompressionProfileState {
  profiles: CompressionProfileView[];
  effective: ResolvedCompressionProfileView | null;
  compressionAvailable: boolean;
  select(profileId: string): Promise<boolean>;
}

export function useSessionCompressionProfile(
  sessionId?: string,
): SessionCompressionProfileState {
  const [profiles, setProfiles] = useState<CompressionProfileView[]>([]);
  const [effective, setEffective] = useState<ResolvedCompressionProfileView | null>(null);

  const refreshEffective = useCallback(async () => {
    if (!sessionId) {
      setEffective(null);
      return;
    }
    try {
      setEffective(await invoke<ResolvedCompressionProfileView>(
        "get_session_compression_profile",
        { sessionId },
      ));
    } catch {
      // Une actualisation transitoire ne remplace jamais le dernier état valide.
    }
  }, [sessionId]);

  const refreshAll = useCallback(async () => {
    if (!sessionId) {
      setProfiles([]);
      setEffective(null);
      return;
    }
    try {
      const view = await invoke<CompressionProfilesView>("get_compression_profiles");
      setProfiles(view.profiles.slice(0, 20));
    } catch {
      // Le menu reste sur sa dernière liste confirmée.
    }
    await refreshEffective();
  }, [refreshEffective, sessionId]);
  const refreshProfilesEvent = useCallback(() => { void refreshAll(); }, [refreshAll]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- chargement depuis les autorités persistées
    void refreshAll();
  }, [refreshAll]);

  useEffect(() => {
    if (!sessionId) return;
    const refresh = () => { void refreshEffective(); };
    window.addEventListener(AGENT_SESSIONS_CHANGED, refresh);
    const sessionEvent = listen("agent-session-updated", refresh);
    const modelfileEvent = listen("modelfile-updated", refresh);
    return () => {
      window.removeEventListener(AGENT_SESSIONS_CHANGED, refresh);
      cleanupTauriListener(sessionEvent);
      cleanupTauriListener(modelfileEvent);
    };
  }, [refreshEffective, sessionId]);

  useFsEvent("fs:compression-profiles-changed", refreshProfilesEvent);

  const select = useCallback(async (profileId: string): Promise<boolean> => {
    if (!sessionId) return false;
    try {
      const next = await invoke<ResolvedCompressionProfileView>(
        "set_session_compression_profile",
        { sessionId, profileId },
      );
      setEffective(next);
      notifyAgentSessionsChanged();
      return true;
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
      return false;
    }
  }, [sessionId]);

  return {
    profiles,
    effective,
    compressionAvailable: effective?.available ?? false,
    select,
  };
}
