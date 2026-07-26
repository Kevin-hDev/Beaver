export interface AgentMessage {
  id: string;
  role: "user" | "assistant" | "tool";
  content: string;
  thinking?: string;
  tool_calls?: ToolCallRequest[];
  tool_name?: string;
  tool_activities?: ToolActivityRecord[];
  segments?: SavedSegment[];
  files: FileAttachment[];
  timestamp: string;
  skill_names?: string[];
  tokens?: number;
  work_duration_ms?: number;
  stream_run_id?: string;
  stream_part?: StreamMessagePart;
  /** Marqueur frontend temporaire : ce bloc appartient encore au stream actif. */
  is_stream_checkpoint?: boolean;
}

export type StreamMessagePart = "checkpoint" | "input" | "final";

export interface SavedSegment {
  thinking?: string;
  tools: ToolActivityRecord[];
  content: string;
  phase?: "work" | "final";
}

export interface ToolActivityRecord {
  name: string;
  summary: string;
  domain?: "memory";
  args?: Record<string, unknown>;
  result?: string;
  is_error?: boolean;
  content?: string;
  old_text?: string;
  new_text?: string;
  start_line?: number;
  resolved_path?: string;
  affected_paths?: string[];
  file_changes?: ToolFileChangeRecord[];
}

export interface ToolFileChangeRecord {
  path: string;
  status: "added" | "modified" | "deleted";
  additions: number;
  deletions: number;
  diff?: GitDiffPreview;
}

export interface ToolCallRequest {
  extra_content?: unknown;
  function: { name: string; arguments: Record<string, unknown> };
}

export interface FileAttachment {
  name: string;
  path: string;
  mime_type: string;
  size: number;
  thumbnail?: string;
  access_grant?: string;
}

export interface SkillInfo {
  id: string;
  name: string;
  command: string;
  description: string;
  path: string;
  source: string;
  source_name: string;
}
import type { GitDiffPreview } from "./file-preview";
