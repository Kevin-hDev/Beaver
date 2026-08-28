import { buildSegmentedMessage, type StreamSegment } from "./agent-chat-utils";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import type { ManagedStreamState } from "./agent-chat-stream-types";
import type { AgentMessage } from "@/types/agent";

export function visibleAssistant(
  state: ManagedStreamState,
  outputTokens: number | null,
  terminalResponse: boolean,
): AgentMessage | undefined {
  const segments = visibleSegments(state);
  if (segments.length === 0) return undefined;
  const built = buildSegmentedMessage(segments);
  const totalMs = state.streamStartedAt ? Date.now() - state.streamStartedAt : 0;
  const message: AgentMessage = {
    id: state.activeTurn?.assistantMessageId ?? crypto.randomUUID(),
    turn_id: state.activeTurn?.turnId,
    role: "assistant",
    content: built.content,
    thinking: built.thinking,
    tool_activities: built.toolRecords,
    segments: built.segments,
    files: [],
    timestamp: new Date().toISOString(),
    tokens: 0,
    work_duration_ms: totalMs > 0 ? totalMs : undefined,
    stream_run_id: state.streamRunId,
    stream_part: terminalResponse ? "final" : "checkpoint",
  };
  message.tokens = outputTokens ?? estimateAgentMessagesTokens([message]);
  return message;
}

export function visibleSegments(state: ManagedStreamState): StreamSegment[] {
  if (!state.currentContent && !state.currentThinking && state.currentTools.length === 0) {
    return state.completedSegments;
  }
  return [...state.completedSegments, {
    thinking: state.currentThinking,
    tools: state.currentTools,
    content: state.currentContent,
    phase: state.currentContentPhase,
  }];
}
