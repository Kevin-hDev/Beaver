export type WakeupSchedule =
  | { kind: "once"; local_datetime: string; timezone: string }
  | { kind: "cron"; expression: string; timezone: string }
  | { kind: "after_completion"; delay_minutes: number };

export type WakeupTarget =
  | { mode: "new_session"; project_id?: string | null }
  | { mode: "resume_session"; session_id: string };

export type WakeupStatus = "active" | "disabled" | "completed";
export type WakeupDisplayStatus = WakeupStatus | "running" | "paused_by_global";

export interface WakeupLastRun {
  status: WakeupRunStatus;
  finished_at: string;
  error_code?: WakeupRunErrorCode | null;
}

export interface ScheduledWakeup {
  id: string;
  revision: number;
  name: string;
  model: string;
  provider: string;
  target: WakeupTarget;
  schedule: WakeupSchedule;
  status: WakeupStatus;
  running: boolean;
  paused_by_global: boolean;
  next_fire_at: string | null;
  last_run: WakeupLastRun | null;
}

export interface WakeupDefinition extends Omit<ScheduledWakeup, "running" | "paused_by_global" | "next_fire_at" | "last_run"> {
  description: string | null;
  prompt: string;
  creator_session_id: string | null;
  created_at: string;
  anchor_at: string | null;
}

export interface WakeupDetail {
  definition: WakeupDefinition;
  next_fire_at: string | null;
}

export interface CreateWakeupInput {
  name: string;
  model: string;
  provider: string;
  prompt: string;
  schedule: WakeupSchedule;
  description?: string;
  project_id?: string;
}

export interface UpdateWakeupInput {
  automation_id: string;
  name?: string;
  model?: string;
  prompt?: string;
  description?: string | null;
  schedule?: WakeupSchedule;
  status?: WakeupStatus;
}

export interface HeartbeatConfig {
  global_paused: boolean;
}

export type WakeupRunStatus = "ok" | "error" | "missed" | "cancelled" | "interrupted";

export type WakeupRunErrorCode =
  | "failed"
  | "rate_limited"
  | "authentication_failed"
  | "ollama_unavailable"
  | "missed_unavailable"
  | "scheduler_stopping"
  | "capacity_reached"
  | "app_stopped"
  | "target_session_missing"
  | "provider_unavailable"
  | "model_unavailable";

export interface WakeupRun {
  run_id?: string | null;
  automation_id: string;
  scheduled_for: string;
  started_at?: string | null;
  finished_at: string;
  status: WakeupRunStatus;
  error_code?: WakeupRunErrorCode;
  error?: string;
  session_id?: string;
  tokens?: number;
  missed_count?: number;
}

export interface WakeupHistoryPage {
  entries: WakeupRun[];
  next_cursor: string | null;
}

export interface MigrationConflict {
  legacy_id: string;
}

export type AutomationMigrationStatus =
  | { status: "ready" }
  | { status: "needs_timezone" }
  | { status: "unavailable" }
  | { status: "conflicts"; conflicts: MigrationConflict[] };

export type AutomationErrorCode =
  | "audit_unavailable"
  | "store_unavailable"
  | "migration_unavailable"
  | "invalid_timezone"
  | "invalid_schedule"
  | "model_unavailable"
  | "provider_unavailable"
  | "not_found"
  | "globally_paused"
  | "invalid_project"
  | "invalid_input";

export function displayStatus(wakeup: ScheduledWakeup): WakeupDisplayStatus {
  if (wakeup.running) return "running";
  if (wakeup.paused_by_global) return "paused_by_global";
  return wakeup.status;
}
