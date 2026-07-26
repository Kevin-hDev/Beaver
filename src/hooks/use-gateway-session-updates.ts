import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { RefObject } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { AgentSession } from "@/types/agent";

type GatewaySessionEvent = { session_id: string };

export function listenGatewaySessionUpdates(
  sessionId: string,
  sessionRef: RefObject<string | null>,
  onReload: (session: AgentSession) => void,
): () => void {
  const reloadSession = (e: { payload?: GatewaySessionEvent }) => {
    if (e.payload?.session_id !== sessionId || sessionRef.current !== sessionId) return;
    void invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then(onReload)
      .catch((e: unknown) => console.warn("Session reload:", e));
  };
  const unlisteners = ["wakeup-completed", "agent-session-updated"].map((eventName) =>
    listen<GatewaySessionEvent>(eventName, reloadSession),
  );
  return () => {
    unlisteners.forEach((unlisten) => {
      cleanupTauriListener(unlisten);
    });
  };
}
