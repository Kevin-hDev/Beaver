import type {
  AgentMessageView,
  FileAttachmentView,
  SavedSegmentView,
  ToolActivityRecordView,
  ToolCallRequestView,
  ToolFileChangeView,
} from "./agent-session.generated";

export interface AgentMessage extends Omit<
  AgentMessageView,
  "reasoning_replay_status" | "tokens" | "tool_calls" | "turn_id"
> {
  /** Absent only on messages still being built locally before Rust admission. */
  turn_id?: string;
  tokens?: number;
  tool_calls?: ToolCallRequest[];
  reasoning_replay_status?: AgentMessageView["reasoning_replay_status"];
  /** Marqueur frontend temporaire : ce bloc appartient encore au stream actif. */
  is_stream_checkpoint?: boolean;
}

export type StreamMessagePart = NonNullable<AgentMessageView["stream_part"]>;
export type SavedSegment = SavedSegmentView;
export type ToolActivityRecord = ToolActivityRecordView;
export type ToolFileChangeRecord = ToolFileChangeView;

export interface ToolCallRequest extends Omit<ToolCallRequestView, "id"> {
  id?: string;
}

export type FileAttachment = FileAttachmentView;

export interface SkillInfo {
  id: string;
  name: string;
  command: string;
  description: string;
  path: string;
  source: string;
  source_name: string;
}
