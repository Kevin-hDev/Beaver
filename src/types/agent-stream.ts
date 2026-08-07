import type { AgentInteractiveChoiceRequest } from "./agent-interactive";
import type { AgentMessage } from "./agent-message";
import type { ToolFileChangeRecord } from "./agent-message";
import type { AgentPlanPreview } from "./agent-plan";
import type { SubagentStatus } from "./agent-session";
import type { AgentTodoItem } from "./agent-todo";
import type { ToolErrorInfo, ToolResultStatus } from "./agent-tool-result";

interface AgentErrorDiagnosticSummary {
  requestId: string;
  phase: string;
  errorType: string;
  lastToolName?: string;
  safeSummary: string;
}

export interface ContextCapacityDetails {
  systemTokens: number;
  requiredReportTokens: number;
  toolTokens: number;
  requiredTokens: number;
  maxInputTokens: number;
  contextWindow: number;
}

export type TokenPhase = "work" | "final";

export interface RetryIndicatorState {
  reasonKey: string;
  attempt: number;
  maxAttempts: number;
}

export interface StreamContextUsageBreakdown {
  messages: number;
  systemTools: number;
  mcpConnectors: number;
  skills: number;
  memory: number;
  metaContext: number;
  systemPrompt: number;
  reasoningIncluded: boolean;
}

export type StreamEvent =
  | { event: "token"; data: { content: string; tokenCount: number; tps: number; phase?: TokenPhase } }
  | { event: "contentPhase"; data: { phase: TokenPhase } }
  | { event: "thinking"; data: { content: string; tokenCount?: number } }
  | { event: "contextUsage"; data: { inputTokens: number; outputTokens: number; contextLimit: number; estimated: boolean; breakdown?: StreamContextUsageBreakdown } }
  | { event: "generationStarted"; data: Record<string, never> }
  | { event: "toolCall"; data: { name: string; arguments: Record<string, unknown>; toolCallIndex?: number; toolCallId?: string; domain?: "memory" } }
  | { event: "toolOutput"; data: { toolCallIndex: number; content: string; elapsedMs: number } }
  | { event: "toolResult"; data: { name: string; content: string; isError: boolean; status?: ToolResultStatus; error?: ToolErrorInfo; warnings?: string[]; truncated?: boolean; displaySummary?: string; toolCallIndex: number; toolCallId?: string; resolvedPath?: string; domain?: "memory"; affectedPaths?: string[]; fileChanges?: ToolFileChangeRecord[]; startLine?: number } }
  | { event: "turnEnd"; data: Record<string, never> }
  | { event: "permissionRequest"; data: { id: string; toolName: string; arguments: Record<string, unknown> } }
  | { event: "done"; data: { evalCount: number | null; evalDurationNs: number; finalTps: number; tpsEstimated?: boolean; promptTokens: number | null; contextTokens: number | null } }
  | {
    event: "error";
    data: {
      message: string;
      isConnection?: boolean;
      diagnostic?: AgentErrorDiagnosticSummary;
      contextCapacity?: ContextCapacityDetails;
    };
  }
  | { event: "notice"; data: { messageKey: string } }
  | { event: "retryIndicator"; data: RetryIndicatorState }
  | { event: "compressing"; data: { status: string } }
  | { event: "compressionComplete"; data: Record<string, never> }
  | { event: "sessionSnapshot"; data: { messages: AgentMessage[]; tokenCount: number } }
  | { event: "subagentSpawned"; data: { subagentSessionId: string; subagentName: string; subagentType: string; subagentDescription: string; subagentColorKey: string; promptPreview: string; runId?: string } }
  | { event: "subagentCompleted"; data: { subagentSessionId: string; success: boolean; status: SubagentStatus; summary: string; runId?: string } }
  | { event: "todoUpdated"; data: { todos: AgentTodoItem[] } }
  | { event: "planPreviewUpdated"; data: { plan: AgentPlanPreview | null } }
  | { event: "planModeUpdated"; data: { enabled: boolean } }
  | { event: "interactiveChoiceRequest"; data: AgentInteractiveChoiceRequest };
