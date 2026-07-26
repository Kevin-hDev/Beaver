export type WakeupSchedule =
  | { kind: "once"; datetime: string }
  | { kind: "daily"; time: string }
  | { kind: "weekly"; weekday: number; time: string };

export interface ScheduledWakeup {
  id: string;
  name: string;
  model: string;
  provider: string;
  prompt: string;
  schedule: WakeupSchedule;
  description: string;
  active: boolean;
  paused_by_global: boolean;
  created_at: string;
}

export interface CreateWakeupInput {
  name: string;
  model: string;
  provider: string;
  prompt: string;
  schedule: WakeupSchedule;
  description: string;
}

export interface HeartbeatConfig {
  global_paused: boolean;
}

export type WakeupRunStatus = "ok" | "error" | "missed";

export interface WakeupRun {
  wakeup_id: string;
  scheduled_for: string;
  fired_at: string;
  status: WakeupRunStatus;
  error?: string;
  session_id?: string;
  tokens?: number;
}

export interface WakeupStatusSummary {
  wakeup_id: string;
  next_fire_at: string | null;
  last_run: WakeupRun | null;
}
