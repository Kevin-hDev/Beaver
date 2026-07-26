import { buildSegmentedMessage, type StreamSegment } from "./agent-chat-utils";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import { MAX_MESSAGES_PER_SESSION, type ManagedStreamState, type StreamApplyResult } from "./agent-chat-stream-types";
import type { AgentMessage } from "@/types/agent";

export function checkpointQueuedUserMessages(
  state: ManagedStreamState,
): StreamApplyResult | null {
  if (state.queuedUserMessages.length === 0) return null;
  const segments = currentSegments(state);
  const assistantMessage = segments.length > 0 ? buildAssistant(segments, state) : undefined;
  const persisted = assistantMessage
    ? [assistantMessage, ...state.queuedUserMessages]
    : [...state.queuedUserMessages];
  const messages = trimMessages([...state.messages, ...persisted]);
  return {
    state: {
      ...state,
      messages,
      queuedUserMessages: [],
      completedSegments: [],
      currentContent: "",
      currentContentPhase: undefined,
      currentThinking: "",
      currentTools: [],
      activeStreamItem: null,
      streamStartedAt: state.streamStartedAt,
      segmentStartedAt: Date.now(),
      sessionTokenCount: estimateAgentMessagesTokens(messages),
      sessionTokenCountEstimated: true,
      lastRequestTokens: assistantMessage?.tokens ?? 0,
    },
    assistantMessage,
    assistantTokens: assistantMessage?.tokens ?? 0,
    messagesToPersist: persisted,
  };
}

function currentSegments(state: ManagedStreamState): StreamSegment[] {
  if (!state.currentThinking && !state.currentContent && state.currentTools.length === 0) {
    return state.completedSegments;
  }
  return [...state.completedSegments, {
    thinking: state.currentThinking,
    tools: state.currentTools,
    content: state.currentContent,
    phase: state.currentContentPhase,
  }];
}

function buildAssistant(segments: StreamSegment[], state: ManagedStreamState): AgentMessage {
  const built = buildSegmentedMessage(segments);
  const elapsed = state.streamStartedAt ? Date.now() - state.streamStartedAt : 0;
  const message: AgentMessage = {
    id: crypto.randomUUID(), role: "assistant", content: built.content,
    thinking: built.thinking, tool_activities: built.toolRecords,
    segments: built.segments, files: [], timestamp: new Date().toISOString(),
    work_duration_ms: elapsed > 0 ? elapsed : undefined, tokens: 0,
    stream_run_id: state.streamRunId,
    stream_part: "checkpoint",
    is_stream_checkpoint: true,
  };
  message.tokens = estimateAgentMessagesTokens([message]);
  return message;
}

function trimMessages(messages: AgentMessage[]): AgentMessage[] {
  if (messages.length <= MAX_MESSAGES_PER_SESSION) return messages;
  return messages.slice(messages.length - MAX_MESSAGES_PER_SESSION);
}
