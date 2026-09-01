import type { AgentLocalNavState, AgentLocalWorkspaceState } from "@/types/navigation";

export interface AgentLocalTabProps {
  navState: AgentLocalNavState;
  onSessionChange?: (id: string | null) => void;
  onNavChange?: (partial: Partial<AgentLocalWorkspaceState>) => void;
  onWorkspaceClear?: (sessionId: string) => void;
  listFocused?: boolean;
}
