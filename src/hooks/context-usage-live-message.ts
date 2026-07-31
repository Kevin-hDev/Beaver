import { buildSegmentedMessage, type StreamSegment, type ToolActivity } from "./agent-chat-utils";
import type { AgentMessage, TokenPhase } from "@/types/agent";

export interface LiveContextState {
  completedSegments: StreamSegment[];
  currentContent: string;
  currentContentPhase?: TokenPhase;
  currentThinking: string;
  currentTools: ToolActivity[];
}

export function buildLiveContextMessage(
  state: LiveContextState,
): AgentMessage | null {
  const segments = [...state.completedSegments];
  if (state.currentContent || state.currentThinking || state.currentTools.length > 0) {
    segments.push({
      content: state.currentContent,
      thinking: state.currentThinking,
      tools: state.currentTools,
      phase: state.currentContentPhase,
    });
  }
  if (segments.length === 0) return null;

  const built = buildSegmentedMessage(segments);
  return {
    id: "context-usage-live",
    role: "assistant",
    content: built.content,
    thinking: built.thinking,
    tool_activities: built.toolRecords,
    segments: built.segments,
    files: [],
    timestamp: "",
    tokens: 0,
  };
}
