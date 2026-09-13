import type { StreamSegment, ToolActivity } from "./agent-chat-utils";
import type {
  AgentInteractiveChoiceRequest,
  AgentMessage,
  AgentPlanPreview,
  RetryIndicatorState,
  TokenPhase,
} from "@/types/agent";
import type { AgentPermissionRequest } from "@/types/agent-stream";
import type { ActiveStreamItem } from "./active-stream-item";
import type { ContextTokenBuckets } from "./context-usage-buckets";
import type { ContextUsageRecord } from "@/types/agent-session.generated";
import { EMPTY_CONTEXT_USAGE_RECORD } from "./agent-token-estimate";

export interface VisibleTurnIdentity {
  turnId: string;
  userMessageId: string;
  assistantMessageId: string;
}

export const MAX_PENDING_PERMISSIONS = 32;
export const MAX_MESSAGES_PER_SESSION = 2000;
export const MAX_QUEUED_USER_MESSAGES = 8;

export type StreamKind = "chat" | "compression";

export interface ChatState {
  messages: AgentMessage[];
  queuedUserMessages: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentContentPhase?: TokenPhase;
  currentThinking: string;
  currentTools: ToolActivity[];
  activeStreamItem: ActiveStreamItem;
  isStreaming: boolean;
  // The visible work ends at turnCommitted; stream ownership stays active until done.
  isWorking: boolean;
  isCompressing: boolean;
  tps: number;
  tpsEstimated: boolean;
  sessionTokenCount: number;
  contextUsageRecord: ContextUsageRecord;
  contextLimitTokens: number;
  contextUsageBuckets: ContextTokenBuckets | null;
  contextUsageBaseSegments: number;
  contextUsageIncludesReasoning: boolean;
  contextUsageVisible: boolean;
  liveTokenCount: number;
  requestOutputTokens: number;
  streamStartedAt: number | null;
  segmentStartedAt: number | null;
  totalElapsedMs: number;
  streamRunId: string;
  error?: string;
  isConnectionError?: boolean;
  diagnosticSummary?: string;
  retryIndicator?: RetryIndicatorState | null;
  interactiveChoice?: AgentInteractiveChoiceRequest;
  planPreview?: AgentPlanPreview | null;
  planModeEnabled?: boolean;
}

export type PermissionRequestState = AgentPermissionRequest;

export interface ManagedStreamState extends ChatState {
  pendingPermissions: PermissionRequestState[];
  activeTurn?: VisibleTurnIdentity;
  completed: boolean; updatedAt: number; error?: string; isConnectionError?: boolean; diagnosticSummary?: string;
}

export const EMPTY_CHAT_STATE: ChatState = {
  messages: [], queuedUserMessages: [], completedSegments: [], currentContent: "",
  currentContentPhase: undefined, currentThinking: "", currentTools: [],
  activeStreamItem: null, isStreaming: false, isWorking: false, isCompressing: false,
  tps: 0, tpsEstimated: false, sessionTokenCount: 0,
  contextUsageRecord: EMPTY_CONTEXT_USAGE_RECORD, contextLimitTokens: 0,
  contextUsageBuckets: null, contextUsageBaseSegments: 0,
  contextUsageIncludesReasoning: true, contextUsageVisible: false,
  liveTokenCount: 0, requestOutputTokens: 0,
  streamStartedAt: null, segmentStartedAt: null,
  totalElapsedMs: 0,
  streamRunId: "",
};

export interface StreamApplyResult {
  state: ManagedStreamState;
  assistantMessage?: AgentMessage;
  assistantTokens?: number;
}

export function createManagedStreamState(
  messages: AgentMessage[],
  sessionTokenCount: number,
  streamKind: StreamKind = "chat",
  contextUsageRecord: ContextUsageRecord = EMPTY_CONTEXT_USAGE_RECORD,
): ManagedStreamState {
  const now = Date.now();
  return {
    ...EMPTY_CHAT_STATE, messages, sessionTokenCount, contextUsageRecord,
    contextUsageVisible: messages.some((message) => message.role === "assistant"),
    isStreaming: true,
    isWorking: true,
    isCompressing: streamKind === "compression",
    streamRunId: crypto.randomUUID(),
    streamStartedAt: now, segmentStartedAt: now,
    pendingPermissions: [], completed: false,
    updatedAt: now,
  };
}

export function toChatState(state: ManagedStreamState): ChatState {
  return {
    messages: state.messages, queuedUserMessages: state.queuedUserMessages,
    completedSegments: state.completedSegments,
    currentContent: state.currentContent, currentContentPhase: state.currentContentPhase,
    currentThinking: state.currentThinking,
    currentTools: state.currentTools, activeStreamItem: state.activeStreamItem,
    isStreaming: state.isStreaming,
    isWorking: state.isWorking,
    isCompressing: state.isCompressing,
    tps: state.tps, tpsEstimated: state.tpsEstimated,
    sessionTokenCount: state.sessionTokenCount,
    contextUsageRecord: state.contextUsageRecord,
    contextLimitTokens: state.contextLimitTokens,
    contextUsageBuckets: state.contextUsageBuckets,
    contextUsageBaseSegments: state.contextUsageBaseSegments,
    contextUsageIncludesReasoning: state.contextUsageIncludesReasoning,
    contextUsageVisible: state.contextUsageVisible,
    liveTokenCount: state.liveTokenCount,
    requestOutputTokens: state.requestOutputTokens,
    streamStartedAt: state.streamStartedAt,
    segmentStartedAt: state.segmentStartedAt,
    totalElapsedMs: state.totalElapsedMs,
    streamRunId: state.streamRunId,
    error: state.error,
    isConnectionError: state.isConnectionError,
    diagnosticSummary: state.diagnosticSummary,
    retryIndicator: state.retryIndicator,
    interactiveChoice: state.interactiveChoice,
    planPreview: state.planPreview,
    planModeEnabled: state.planModeEnabled,
  };
}
