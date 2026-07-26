import { buildSegmentedMessage, type StreamSegment } from "./agent-chat-utils";
import { markUnconfirmedContentAsWork } from "./agent-chat-stream-partial";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import {
  MAX_MESSAGES_PER_SESSION,
  type ManagedStreamState,
  type StreamApplyResult,
} from "./agent-chat-stream-types";
import type { AgentMessage, StreamEvent } from "@/types/agent";

export function finishPartialStream(state: ManagedStreamState): StreamApplyResult {
  return finalizeStream(
    markPendingToolsCancelled(markUnconfirmedContentAsWork(state)), null, state.tps, null, false,
  );
}

export function finishStream(
  state: ManagedStreamState,
  event: Extract<StreamEvent, { event: "done" }>,
) {
  return finalizeStream(
    state, event.data.evalCount, event.data.finalTps, event.data.contextTokens, true,
  );
}

export function finalizeStream(
  state: ManagedStreamState,
  outputTokens: number | null,
  tps: number,
  contextTokens: number | null,
  terminalResponse: boolean,
): StreamApplyResult {
  const segments = streamSegments(state);
  const totalMs = state.streamStartedAt ? Date.now() - state.streamStartedAt : 0;
  const assistantMessage = buildAssistant(
    segments, totalMs, outputTokens, state.streamRunId, terminalResponse,
  );
  const persistedMessages = assistantMessage
    ? [assistantMessage, ...state.queuedUserMessages]
    : [...state.queuedUserMessages];
  const allMessages = trimMessages([...state.messages, ...persistedMessages]);
  const hasRealContextTokens = contextTokens !== null;
  const next: ManagedStreamState = {
    ...state,
    messages: allMessages,
    queuedUserMessages: [],
    completedSegments: [], currentContent: "", currentThinking: "",
    currentContentPhase: undefined, currentTools: [], activeStreamItem: null,
    isStreaming: false, isCompressing: false, tps,
    sessionTokenCount: contextTokens ?? estimateAgentMessagesTokens(allMessages),
    sessionTokenCountEstimated: !hasRealContextTokens,
    lastRequestTokens: assistantMessage?.tokens ?? outputTokens ?? 0, liveTokenCount: 0,
    streamStartedAt: null, segmentStartedAt: null, totalElapsedMs: totalMs,
    pendingPermissions: [], interactiveChoice: undefined,
    completed: true, updatedAt: Date.now(),
  };
  if (segments.length === 0 && persistedMessages.length === 0) return { state: next };
  return {
    state: next,
    assistantMessage,
    assistantTokens: assistantMessage?.tokens ?? outputTokens ?? 0,
    messagesToPersist: persistedMessages,
  };
}

function streamSegments(state: ManagedStreamState): StreamSegment[] {
  if (!state.currentContent && !state.currentThinking && state.currentTools.length === 0) {
    return state.completedSegments;
  }
  return [...state.completedSegments, {
    thinking: state.currentThinking, tools: state.currentTools, content: state.currentContent,
    phase: state.currentContentPhase,
  }];
}

function buildAssistant(
  segments: StreamSegment[], totalMs: number, outputTokens: number | null,
  streamRunId: string, terminalResponse: boolean,
): AgentMessage | undefined {
  if (segments.length === 0) return undefined;
  const built = buildSegmentedMessage(segments);
  const message: AgentMessage = {
    id: crypto.randomUUID(), role: "assistant", content: built.content,
    thinking: built.thinking, tool_activities: built.toolRecords,
    segments: built.segments, files: [], timestamp: new Date().toISOString(), tokens: 0,
    work_duration_ms: totalMs > 0 ? totalMs : undefined,
    stream_run_id: streamRunId,
    stream_part: terminalResponse ? "final" : "checkpoint",
  };
  message.tokens = outputTokens ?? estimateAgentMessagesTokens([message]);
  return message;
}

function markPendingToolsCancelled(state: ManagedStreamState): ManagedStreamState {
  if (state.currentTools.every((tool) => tool.result)) return state;
  return {
    ...state,
    currentTools: state.currentTools.map((tool) => tool.result
      ? tool
      : { ...tool, result: "Annulé.", isError: true }),
    activeStreamItem: null,
  };
}

function trimMessages(messages: AgentMessage[]) {
  return messages.length > MAX_MESSAGES_PER_SESSION
    ? messages.slice(messages.length - MAX_MESSAGES_PER_SESSION)
    : messages;
}
