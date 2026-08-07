import type { StreamSegment } from "./agent-chat-utils";
import type { StreamEvent, TokenPhase } from "@/types/agent";
import i18n from "@/i18n";
import { isHiddenAgentTool } from "@/lib/hidden-agent-tools";
import {
  MAX_PENDING_PERMISSIONS, KNOWN_ERROR_KEYS,
  type ChatState, type ManagedStreamState, type StreamApplyResult,
  type PermissionRequestState,
} from "./agent-chat-stream-types";
import { activeItemAfterToolResult, pendingToolIndices, thinkingItem, toolItems } from "./active-stream-item";
import { applyToolResult } from "./agent-chat-tool-results";
import { checkpointQueuedUserMessages } from "./agent-stream-user-checkpoint";
import { finishInterruptedStream, finishStream } from "./agent-chat-stream-finalize";
import { applyContextUsage, applyGeneratedTokenCount } from "./agent-stream-context-usage";
import { applyToolOutput } from "./agent-chat-stream-tool-output";
import { applyRetryIndicator } from "./agent-chat-stream-retry";
import { contextCapacityErrorMessage } from "./agent-context-capacity-error";

export type { ChatState, ManagedStreamState, PermissionRequestState, StreamApplyResult };
export { EMPTY_CHAT_STATE, createManagedStreamState, toChatState } from "./agent-chat-stream-types";
export { finishPartialStream } from "./agent-chat-stream-finalize";
export { clearInteractiveChoiceState } from "./agent-chat-interactive-state";

export function applyStreamEvent(
  state: ManagedStreamState,
  event: StreamEvent,
): StreamApplyResult {
  const now = Date.now();
  const next = { ...state, updatedAt: now };
  const ensureTimers = () => {
    if (!next.streamStartedAt) next.streamStartedAt = now;
    if (!next.segmentStartedAt) next.segmentStartedAt = now;
  };
  switch (event.event) {
    case "token":
      ensureTimers();
      next.contextUsageVisible = true;
      next.retryIndicator = null;
      next.activeStreamItem = null;
      if (event.data.phase) prepareContentPhase(next, event.data.phase);
      next.currentContent += event.data.content;
      next.tps = event.data.tps;
      next.tpsEstimated = true;
      applyGeneratedTokenCount(next, event.data.tokenCount);
      break;
    case "contentPhase":
      ensureTimers();
      prepareContentPhase(next, event.data.phase);
      break;
    case "thinking":
      ensureTimers();
      next.contextUsageVisible = true;
      next.currentThinking += event.data.content;
      next.activeStreamItem = thinkingItem();
      applyGeneratedTokenCount(next, event.data.tokenCount);
      break;
    case "contextUsage":
      applyContextUsage(next, event.data);
      break;
    case "generationStarted":
      next.contextUsageVisible = true;
      break;
    case "toolCall":
      ensureTimers();
      next.contextUsageVisible = true;
      if (isHiddenAgentTool(event.data.name)) break;
      next.currentTools = [...next.currentTools, {
        name: event.data.name, args: event.data.arguments, domain: event.data.domain,
        callIndex: event.data.toolCallIndex,
        callId: event.data.toolCallId,
      }];
      next.activeStreamItem = toolItems(pendingToolIndices(next.currentTools));
      break;
    case "toolOutput":
      applyToolOutput(next, event.data);
      break;
    case "toolResult": {
      if (isHiddenAgentTool(event.data.name)) {
        next.pendingPermissions = [];
        if (event.data.name === "ask_user_choice" || event.data.name === "plan_mode") {
          next.interactiveChoice = undefined;
        }
        break;
      }
      const applied = applyToolResult(next.currentTools, {
        name: event.data.name,
        callIndex: event.data.toolCallIndex ?? -1,
        callId: event.data.toolCallId,
        content: event.data.content,
        isError: event.data.isError,
        status: event.data.status,
        error: event.data.error,
        warnings: event.data.warnings,
        truncated: event.data.truncated,
        resolvedPath: event.data.resolvedPath,
        domain: event.data.domain,
        affectedPaths: event.data.affectedPaths,
        fileChanges: event.data.fileChanges,
        startLine: event.data.startLine,
        displaySummary: event.data.displaySummary,
      });
      next.currentTools = applied.tools;
      next.activeStreamItem = activeItemAfterToolResult(
        next.currentTools,
        applied.appliedIndex,
      );
      next.pendingPermissions = [];
      break;
    }
    case "turnEnd":
      next.retryIndicator = null;
      {
        const checkpoint = checkpointQueuedUserMessages(next);
        if (checkpoint) return checkpoint;
      }
      next.completedSegments = appendCurrentSegment(next);
      next.currentContent = "";
      next.currentContentPhase = undefined;
      next.currentThinking = "";
      next.currentTools = [];
      next.activeStreamItem = null;
      next.segmentStartedAt = null;
      break;
    case "permissionRequest":
      next.pendingPermissions = addPermission(next.pendingPermissions, { id: event.data.id,
        toolName: event.data.toolName, arguments: event.data.arguments });
      break;
    case "sessionSnapshot":
      break;
    case "subagentSpawned":
    case "subagentCompleted":
    case "todoUpdated":
      break;
    case "planPreviewUpdated":
      next.planPreview = event.data.plan;
      break;
    case "planModeUpdated":
      next.planModeEnabled = event.data.enabled;
      if (!event.data.enabled) next.planPreview = null;
      break;
    case "interactiveChoiceRequest":
      next.interactiveChoice = event.data;
      break;
    case "retryIndicator":
      applyRetryIndicator(next, event.data, now);
      break;
    case "compressing":
      next.isCompressing = event.data.status === "start";
      break;
    case "compressionComplete":
      next.isCompressing = false;
      break;
    case "done":
      next.retryIndicator = null;
      return finishStream(next, event);
    case "error": {
      const rawMsg = event.data.message || "";
      const errorKey = KNOWN_ERROR_KEYS[rawMsg];
      next.error = contextCapacityErrorMessage(rawMsg, event.data.contextCapacity)
        ?? (errorKey ? i18n.t(errorKey) : i18n.t("errors.streamInterrupted"));
      next.isConnectionError = (event.data as Record<string, unknown>).isConnection === true;
      next.diagnosticSummary = event.data.diagnostic?.safeSummary;
      const partial = finishInterruptedStream(next);
      partial.state.error = next.error;
      partial.state.isConnectionError = next.isConnectionError;
      partial.state.diagnosticSummary = next.diagnosticSummary;
      partial.state.retryIndicator = null;
      return partial;
    }
    case "notice":
      break;
  }
  return { state: next };
}

function appendCurrentSegment(state: ChatState): StreamSegment[] {
  if (!state.currentThinking && !state.currentContent && state.currentTools.length === 0) {
    return state.completedSegments;
  }
  return [...state.completedSegments, {
    thinking: state.currentThinking, tools: state.currentTools, content: state.currentContent,
    phase: state.currentContentPhase,
  }];
}

function prepareContentPhase(state: ManagedStreamState, phase: TokenPhase) {
  if (!state.currentContentPhase || state.currentContentPhase === phase) {
    state.currentContentPhase = phase;
    return;
  }
  if (state.currentContent || state.currentThinking || state.currentTools.length > 0) {
    state.completedSegments = appendCurrentSegment(state);
    state.currentContent = "";
    state.currentThinking = "";
    state.currentTools = [];
    state.activeStreamItem = null;
    state.segmentStartedAt = Date.now();
  }
  state.currentContentPhase = phase;
}

function addPermission(
  requests: PermissionRequestState[], request: PermissionRequestState,
): PermissionRequestState[] {
  return [...requests.filter((r) => r.id !== request.id), request].slice(-MAX_PENDING_PERMISSIONS);
}
