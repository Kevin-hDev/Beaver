import { markUnconfirmedContentAsWork } from "./agent-chat-stream-partial";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import { resolvePreparedContextBuckets } from "./context-usage-stream";
import {
  MAX_MESSAGES_PER_SESSION,
  type ManagedStreamState,
  type StreamApplyResult,
} from "./agent-chat-stream-types";
import type { AgentMessage, StreamEvent } from "@/types/agent";
import { cancelledToolError } from "@/lib/tool-result-model";
import type { ToolErrorInfo, ToolResultStatus } from "@/types/agent";
import i18n from "@/i18n";
import { visibleAssistant } from "./agent-stream-visible-assistant";
import type { StreamSegment } from "./agent-chat-utils";

type PendingToolOutcome = "cancelled" | "interrupted" | "missing";

export function finishPartialStream(state: ManagedStreamState): StreamApplyResult {
  return finalizeStream(
    resolvePendingTools(markUnconfirmedContentAsWork(state), "cancelled"),
    null, state.tps, true, null, false,
  );
}

export function finishInterruptedStream(state: ManagedStreamState): StreamApplyResult {
  return finalizeStream(
    resolvePendingTools(state, "interrupted"),
    null, 0, true, null, false,
  );
}

export function finishStream(
  state: ManagedStreamState,
  event: Extract<StreamEvent, { event: "done" }>,
) {
  return finalizeStream(
    resolvePendingTools(state, "missing"),
    event.data.evalCount, event.data.finalTps, event.data.tpsEstimated ?? true,
    event.data.contextTokens, true,
  );
}

export function finalizeStream(
  state: ManagedStreamState,
  outputTokens: number | null,
  tps: number,
  tpsEstimated: boolean,
  contextTokens: number | null,
  terminalResponse: boolean,
): StreamApplyResult {
  const totalMs = state.streamStartedAt ? Date.now() - state.streamStartedAt : 0;
  const assistantMessage = visibleAssistant(state, outputTokens, terminalResponse);
  const allMessages = trimMessages(
    assistantMessage ? [...state.messages, assistantMessage] : state.messages,
  );
  const contextUsageBuckets = resolvePreparedContextBuckets(
    state,
    state.queuedUserMessages,
  );
  const hasStreamContextTokens = state.hasContextUsageSnapshot;
  const resolvedContextTokens = contextTokens
    ?? (hasStreamContextTokens ? state.sessionTokenCount : estimateAgentMessagesTokens(allMessages));
  const next: ManagedStreamState = {
    ...state,
    messages: allMessages,
    completedSegments: [], currentContent: "", currentThinking: "",
    currentContentPhase: undefined, currentTools: [], activeStreamItem: null,
    isStreaming: false, isWorking: false, isCompressing: false, tps, tpsEstimated,
    sessionTokenCount: resolvedContextTokens,
    contextInputTokens: resolvedContextTokens,
    contextOutputTokens: 0,
    hasContextUsageSnapshot: contextTokens !== null || hasStreamContextTokens,
    contextUsageBuckets,
    contextUsageBaseSegments: 0,
    liveTokenCount: 0,
    streamStartedAt: null, segmentStartedAt: null, totalElapsedMs: totalMs,
    pendingPermissions: [], interactiveChoice: undefined,
    completed: true, updatedAt: Date.now(),
  };
  if (!assistantMessage) return { state: next };
  return {
    state: next,
    assistantMessage,
    assistantTokens: assistantMessage?.tokens ?? outputTokens ?? 0,
  };
}

function resolvePendingTools(
  state: ManagedStreamState,
  outcome: PendingToolOutcome,
): ManagedStreamState {
  const isComplete = (tool: StreamSegment["tools"][number]) =>
    tool.result !== undefined || tool.isError !== undefined;
  const resolve = (tools: StreamSegment["tools"]) => tools.map((tool) => {
    if (isComplete(tool)) return tool;
    const failure = pendingFailure(outcome);
    return {
      ...tool,
      result: tool.liveOutput ? `${tool.liveOutput}\n\n${failure.message}` : failure.message,
      isError: true,
      status: failure.status,
      error: failure.error,
      liveOutput: undefined,
      liveElapsedMs: undefined,
    };
  });
  const completedPending = state.completedSegments.some((segment) =>
    segment.tools.some((tool) => !isComplete(tool)));
  const currentPending = state.currentTools.some((tool) => !isComplete(tool));
  if (!completedPending && !currentPending) return state;
  return {
    ...state,
    completedSegments: state.completedSegments.map((segment) => ({
      ...segment,
      tools: resolve(segment.tools),
    })),
    currentTools: resolve(state.currentTools),
    activeStreamItem: null,
  };
}

function pendingFailure(outcome: PendingToolOutcome): {
  message: string;
  status: ToolResultStatus;
  error: ToolErrorInfo;
} {
  if (outcome === "cancelled") {
    return {
      message: i18n.t("agentLocal.toolActivity.resultCancelled"),
      status: "cancelled",
      error: cancelledToolError(),
    };
  }
  const missing = outcome === "missing";
  return {
    message: i18n.t(missing
      ? "agentLocal.toolActivity.resultMissing"
      : "errors.streamInterrupted"),
    status: "error",
    error: {
      code: missing ? "tool_result_missing" : "tool_result_unavailable",
      category: missing ? "internal" : "unavailable",
      retryable: false,
      hint: i18n.t("agentLocal.toolActivity.verifyBeforeRetry"),
    },
  };
}

function trimMessages(messages: AgentMessage[]) {
  return messages.length > MAX_MESSAGES_PER_SESSION
    ? messages.slice(messages.length - MAX_MESSAGES_PER_SESSION)
    : messages;
}
