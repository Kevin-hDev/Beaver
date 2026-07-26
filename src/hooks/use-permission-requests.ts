import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { agentStreamManager } from "./agent-stream-manager";

const MAX_PERMISSION_QUEUE = 32;

export interface PermissionRequest {
  id: string;
  toolName: string;
  arguments: Record<string, unknown>;
}

export type PermissionDecision = "allow" | "allow_session" | "deny";

export function usePermissionRequests() {
  const [queue, setQueue] = useState<PermissionRequest[]>([]);

  const enqueue = useCallback((req: PermissionRequest) => {
    setQueue((q) => [...q.filter((item) => item.id !== req.id), req].slice(-MAX_PERMISSION_QUEUE));
  }, []);

  const respond = useCallback(async (id: string, decision: PermissionDecision) => {
    try {
      await invoke("respond_to_permission", { id, decision });
    } catch (e) {
      console.error("respond_to_permission:", e);
    }
    agentStreamManager.clearPermission(id);
    setQueue((q) => q.filter((r) => r.id !== id));
  }, []);

  const clear = useCallback(() => setQueue([]), []);

  return {
    queue,
    current: queue[0] ?? null,
    enqueue,
    respond,
    clear,
  };
}
