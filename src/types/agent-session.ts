import type { AgentMessage } from "./agent-message";
import type { AgentPlanApprovalDecision, AgentPlanRun, AgentPlanWorkflowStatus } from "./agent-plan";
import type { AgentTodoItem, AgentTodoRun } from "./agent-todo";

export type CloneMode = "cut" | "summary";
export type SubagentStatus = "running" | "completed" | "failed" | "cancelled" | "interrupted";

interface SubagentLastActivity {
  kind: string;
  label: string;
  detail?: string;
  updated_at: string;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  order: number;
  created_at: string;
}

interface AgentStreamFailure {
  code: string;
  occurred_at: string;
  is_connection: boolean;
  active_todo_run_id?: string;
  active_todo_title?: string;
}

export interface AgentSession {
  id: string;
  name: string;
  created_at: string;
  updated_at?: string;
  archived_at?: string;
  model: string;
  provider: string;
  thinking_enabled: boolean;
  reasoning_mode?: string;
  accumulated_tokens: number;
  messages: AgentMessage[];
  todos?: AgentTodoItem[];
  todo_runs?: AgentTodoRun[];
  active_todo_run_id?: string;
  stream_failures?: AgentStreamFailure[];
  plan_mode_enabled?: boolean;
  plan_runs?: AgentPlanRun[];
  active_plan_id?: string;
  plan_workflow_status?: AgentPlanWorkflowStatus;
  plan_approval_decision?: AgentPlanApprovalDecision;
  project_id?: string;
  working_dir?: string;
  parent_session_id?: string;
  subagent_type?: "explorer" | "coder";
  subagent_worktree?: string;
  subagent_prompt?: string;
  subagent_status?: SubagentStatus;
  subagent_run_id?: string;
  subagent_description?: string;
  subagent_color_key?: string;
  subagent_summary?: string;
  subagent_last_activity?: SubagentLastActivity;
  clone_parent_session_id?: string;
  clone_parent_message_id?: string;
  clone_mode?: CloneMode;
  clone_summary?: string;
  clone_read_files?: string[];
  clone_modified_files?: string[];
  /** Racine du groupe d'onglets (session principale d'origine). Contrairement
   *  à `clone_parent_session_id` (parent immédiat), pointe toujours vers la racine. */
  clone_root_session_id?: string;
  git_branch?: string;
}

export interface AgentSessionMeta {
  id: string;
  name: string;
  created_at: string;
  updated_at?: string;
  archived_at?: string;
  model: string;
  provider: string;
  thinking_enabled?: boolean;
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
