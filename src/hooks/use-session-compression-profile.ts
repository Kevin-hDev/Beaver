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
  profilesStatus: "loading" | "ready" | "error";
  effective: ResolvedCompressionProfileView | null;
  compressionAvailable: boolean;
  select(profileId: string): Promise<boolean>;
}

export function useSessionCompressionProfile(
  sessionId?: string,
): SessionCompressionProfileState {
  const [profiles, setProfiles] = useState<CompressionProfileView[]>([]);
  const [profilesStatus, setProfilesStatus] = useState<"loading" | "ready" | "error">("loading");
  const [effectiveState, setEffectiveState] = useState<{
    sessionId: string;
    value: ResolvedCompressionProfileView;
  } | null>(null);
  const effective = effectiveState && effectiveState.sessionId === sessionId
    ? effectiveState.value
    : null;

  const refreshEffective = useCallback(async () => {
    if (!sessionId) {
      setEffectiveState(null);
      return;
    }
    try {
      const value = await invoke<ResolvedCompressionProfileView>(
        "get_session_compression_profile",
        { sessionId },
      );
      setEffectiveState({ sessionId, value });
    } catch {
      // Une actualisation transitoire ne remplace jamais le dernier état valide.
    }
  }, [sessionId]);

  const refreshAll = useCallback(async () => {
    if (!sessionId) {
      setProfiles([]);
      setProfilesStatus("loading");
      setEffectiveState(null);
      return;
    }
    try {
      const view = await invoke<CompressionProfilesView>("get_compression_profiles");
      setProfiles(view.profiles.slice(0, 20));
      setProfilesStatus("ready");
    } catch {
      setProfilesStatus("error");
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
      setEffectiveState({ sessionId, value: next });
      notifyAgentSessionsChanged();
      return true;
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
      return false;
    }
  }, [sessionId]);

  return {
    profiles,
    profilesStatus,
    effective,
    compressionAvailable: effective?.available ?? false,
    select,
  };
}
