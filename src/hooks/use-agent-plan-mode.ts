import { useCallback, useMemo, useState, type Dispatch, type SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import type { AgentSession } from "@/types/agent";
import type { ChatState } from "./agent-chat-stream-types";

type SetChatState = Dispatch<SetStateAction<ChatState>>;

export function useAgentPlanMode(sessionId: string | null, setChatState: SetChatState) {
  const [enabled, setEnabledState] = useState(false);

  const reset = useCallback(() => setEnabledState(false), []);

  const applySession = useCallback((session: AgentSession) => {
    setEnabledState(session.plan_mode_enabled === true);
  }, []);

  const applyStreamEnabled = useCallback((next: boolean | undefined) => {
    if (next != null) setEnabledState(next);
  }, []);

  const setEnabled = useCallback(async (next: boolean) => {
    if (!sessionId) return;
    setEnabledState(next);
    setChatState((state) => ({
      ...state,
      planModeEnabled: next,
      planPreview: next ? state.planPreview : null,
    }));
    await invoke("set_session_plan_mode", { id: sessionId, enabled: next }).catch(() => {
      setEnabledState(!next);
      setChatState((state) => ({ ...state, planModeEnabled: !next }));
      showToast(i18n.t("errors.sessionSaveFailed"), "error");
    });
  }, [sessionId, setChatState]);

  return useMemo(() => ({
    enabled,
    reset,
    applySession,
    applyStreamEnabled,
    setEnabled,
  }), [enabled, reset, applySession, applyStreamEnabled, setEnabled]);
}
