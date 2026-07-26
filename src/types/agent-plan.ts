type AgentPlanStatus = "draft" | "awaiting_approval" | "approved" | "rejected" | "cancelled";

export type AgentPlanWorkflowStatus =
  | "needs_context"
  | "collecting_questions"
  | "plan_published"
  | "awaiting_approval"
  | "approved"
  | "rejected"
  | "cancelled";

export type AgentPlanApprovalDecision = "implement" | "continue_planning" | "quit_plan";

export interface AgentPlanRun {
  id: string;
  title: string;
  status: AgentPlanStatus;
  path: string;
  created_at: string;
  updated_at: string;
}

export interface AgentPlanPreview {
  id: string;
  title: string;
  content: string;
  status: AgentPlanStatus;
}
