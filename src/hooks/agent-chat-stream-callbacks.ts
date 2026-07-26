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
import { finalizeStream, finishStream } from "./agent-chat-stream-finalize";

export type { ChatState, ManagedStreamState, PermissionRequestState, StreamApplyResult };
export { EMPTY_CHAT_STATE, createManagedStreamState, toChatState } from "./agent-chat-stream-types";
export { finishPartialStream } from "./agent-chat-stream-finalize";

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
      next.retryIndicator = null;
      next.activeStreamItem = null;
      if (event.data.phase) prepareContentPhase(next, event.data.phase);
      next.currentContent += event.data.content;
      next.tps = event.data.tps;
      next.liveTokenCount = event.data.tokenCount || next.liveTokenCount + 1;
      break;
    case "contentPhase":
      ensureTimers();
      prepareContentPhase(next, event.data.phase);
      break;
    case "thinking":
      ensureTimers();
      next.currentThinking += event.data.content;
      next.activeStreamItem = thinkingItem();
      next.liveTokenCount += 1;
      break;
    case "toolCall":
      ensureTimers();
      if (isHiddenAgentTool(event.data.name)) break;
      next.currentTools = [...next.currentTools, {
        name: event.data.name, args: event.data.arguments, domain: event.data.domain,
      }];
      next.activeStreamItem = toolItems(pendingToolIndices(next.currentTools));
      break;
    case "toolResult": {
      if (isHiddenAgentTool(event.data.name)) {
        next.pendingPermissions = [];
        if (event.data.name === "ask_user_choice" || event.data.name === "planmode") {
          next.interactiveChoice = undefined;
        }
        break;
      }
      const toolCallIndex = event.data.toolCallIndex ?? -1;
      next.currentTools = applyToolResult(
        next.currentTools,
        toolCallIndex,
        event.data.content,
        event.data.isError,
        event.data.resolvedPath,
        event.data.domain,
        event.data.affectedPaths,
        event.data.fileChanges,
        event.data.startLine,
        event.data.displaySummary,
      );
      next.activeStreamItem = activeItemAfterToolResult(next.currentTools, toolCallIndex);
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
      next.retryIndicator = event.data;
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
      next.error = errorKey ? i18n.t(errorKey) : i18n.t("errors.streamInterrupted");
      next.isConnectionError = (event.data as Record<string, unknown>).isConnection === true;
      next.diagnosticSummary = event.data.diagnostic?.safeSummary;
      const partial = finalizeStream(next, null, 0, null, false);
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
