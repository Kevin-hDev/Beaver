import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

export interface AgentLocalTabProps {
  navState: AgentLocalNavState;
  onSessionChange?: (id: string | null) => void;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  listFocused?: boolean;
}
