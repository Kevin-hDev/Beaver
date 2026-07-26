type AgentTodoStatus = "pending" | "in_progress" | "completed";
type AgentTodoRunStatus = "active" | "paused" | "completed";

export interface AgentTodoItem {
  content: string;
  active_form?: string;
  status: AgentTodoStatus;
}

export interface AgentTodoRun {
  id: string;
  title: string;
  status: AgentTodoRunStatus;
  todos: AgentTodoItem[];
  created_at: string;
  updated_at: string;
  paused_reason?: string;
}
