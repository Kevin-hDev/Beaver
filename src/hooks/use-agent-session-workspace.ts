import { useCallback, useMemo, useState } from "react";
import {
  DEFAULT_AGENT_LOCAL_WORKSPACE,
  type AgentLocalWorkspaceState,
} from "@/types/navigation";

const MAX_TRACKED_SESSION_WORKSPACES = 32;

interface WorkspaceEntry {
  sessionId: string | null;
  state: AgentLocalWorkspaceState;
}

function currentWorkspace(entries: WorkspaceEntry[], sessionId: string | null): AgentLocalWorkspaceState {
  return entries.find((entry) => entry.sessionId === sessionId)?.state
    ?? DEFAULT_AGENT_LOCAL_WORKSPACE;
}

export function useAgentSessionWorkspace(sessionId: string | null) {
  const [entries, setEntries] = useState<WorkspaceEntry[]>([]);
  const workspace = useMemo(() => currentWorkspace(entries, sessionId), [entries, sessionId]);

  const updateWorkspace = useCallback((partial: Partial<AgentLocalWorkspaceState>) => {
    setEntries((previous) => {
      const current = currentWorkspace(previous, sessionId);
      const next = { ...current, ...partial };
      const retained = previous.filter((entry) => entry.sessionId !== sessionId);
      return [...retained, { sessionId, state: next }].slice(-MAX_TRACKED_SESSION_WORKSPACES);
    });
  }, [sessionId]);

  return { workspace, updateWorkspace };
}
