import type {
  AgentSessionView,
  SubagentLastActivityView,
} from "./agent-session.generated";
import type { AgentMessage } from "./agent-message";

export type CloneMode = "cut" | "summary";
export type SubagentStatus = "running" | "completed" | "failed" | "cancelled" | "interrupted";
type SubagentLastActivity = SubagentLastActivityView;

export interface Project {
  id: string;
  name: string;
  path: string;
  order: number;
  created_at: string;
}

export type AgentSession = Omit<AgentSessionView, "messages" | "subagent_status" | "preserve_reasoning"> & {
  messages: AgentMessage[];
  subagent_status?: SubagentStatus;
  /* Les fixtures et sessions chargées avant Task 17 restent lisibles ; Rust
     pose explicitement Off au chargement et à toute nouvelle écriture. */
  preserve_reasoning?: AgentSessionView["preserve_reasoning"];
};

export interface AgentSessionMeta {
  id: string;
  name: string;
  created_at: string;
  updated_at?: string;
  archived_at?: string;
  pinned_at?: string;
  model: string;
  provider: string;
  thinking_enabled?: boolean;
  fast_mode_enabled: boolean;
  reasoning_mode?: string;
  message_count: number;
  project_id?: string;
  parent_session_id?: string;
  subagent_type?: "explorer" | "coder";
  subagent_status?: SubagentStatus;
  subagent_run_id?: string;
  subagent_description?: string;
  subagent_color_key?: string;
  subagent_summary?: string;
  subagent_last_activity?: SubagentLastActivity;
  clone_parent_session_id?: string;
  clone_parent_message_id?: string;
  clone_mode?: CloneMode;
  clone_root_session_id?: string;
  git_branch?: string;
  is_gateway?: boolean;
  gateway_channel_key?: string;
}

export interface SessionTab {
  tab_id: string;
  session_id: string;
  label: string;
  is_main: boolean;
  clone_parent_session_id?: string;
  clone_parent_message_id?: string;
  clone_mode?: CloneMode;
  git_branch?: string;
}

export interface SessionTabs {
  active_tab_id: string;
  main_checkpoint_branch?: string;
  tabs: SessionTab[];
}

export interface CloneSessionResult {
  root_session_id: string;
  clone_session_id: string;
  operation_id: string;
  tabs: SessionTabs;
}

export interface SubagentInfo {
  sessionId: string;
  name: string;
  type: "explorer" | "coder";
  status: SubagentStatus;
  promptPreview: string;
  description?: string;
  colorKey?: string;
  summary?: string;
  lastActivity?: SubagentLastActivity;
  runId?: string;
  spawnedAt?: number;
}
