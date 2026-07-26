import { Fragment, memo } from "react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import { SavedToolTimeline, StreamToolTimeline } from "./message-tool-timeline";
import { CompressionIndicator } from "./compression-indicator";
import { ContextCompressionMarker } from "./context-compression-marker";
import { PlanPreviewBubble } from "./plan-preview-bubble";
import { StreamEndArtifacts } from "./stream-end-artifacts";
import { LoadingIndicator } from "./working-stats";
import { isCompressionContextOnlyMessage, isCompressionSummaryMessage } from "@/lib/context-messages";
import { planStreamEndArtifacts } from "@/lib/stream-end-artifacts";
import { normalizeSavedToolHistory } from "@/lib/saved-tool-history";
import type { AgentMessage, AgentPlanPreview, SubagentInfo, TokenPhase } from "@/types/agent";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import type { ActiveStreamItem } from "@/hooks/active-stream-item";
import type { FileOperation } from "@/types/file-preview";
import "./chat.css";
import "./messages.css";

interface MessageListProps {
  messages: AgentMessage[];
  queuedUserMessages?: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentContentPhase?: TokenPhase;
  currentThinking: string;
  currentTools: ToolActivity[];
  activeStreamItem?: ActiveStreamItem;
  isStreaming: boolean;
  isCompressing: boolean;
  tps: number;
  totalElapsedMs: number;
  segmentStartedAt: number | null;
  liveTokenCount: number;
  streamRunId?: string;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onCloneMessage?: (messageId: string) => void;
  onFileClick?: (file: {
    name: string;
    path?: string;
    thumbnail?: string;
    access_grant?: string;
  }) => void;
  onFilePreview?: (path: string) => void;
  onFileReview?: (operation: FileOperation) => void;
  projectPath?: string;
  knownSubagents?: SubagentInfo[];
  onOpenSubagent?: (sessionId: string) => void;
  planPreview?: AgentPlanPreview | null;
}

export function MessageList({
  messages, queuedUserMessages = [], completedSegments, currentContent,
  currentContentPhase, currentThinking,
  currentTools, activeStreamItem = null, isStreaming, tps, totalElapsedMs, segmentStartedAt,
  isCompressing, liveTokenCount, onReload, onEdit, onCloneMessage, onFileClick, onFilePreview, onFileReview,
  projectPath, knownSubagents = [], onOpenSubagent, planPreview, streamRunId = "",
}: MessageListProps) {
  const displayMessages = normalizeSavedToolHistory(messages);
  const lastAssistantIdx = displayMessages.map((message) => message.role).lastIndexOf("assistant");
  const streamStartedAt = segmentStartedAt;
  const endArtifacts = planStreamEndArtifacts(displayMessages, isStreaming, streamRunId);

  const artifactsAfter = (messageId: string) => {
    const placement = endArtifacts.get(messageId);
    if (!placement) return null;
    return (
      <StreamEndArtifacts
        messages={placement.messages}
        projectPath={projectPath}
        knownSubagents={knownSubagents}
        onOpenSubagent={onOpenSubagent}
        onFileReview={onFileReview}
      />
    );
  };

  return (
    <>
      {displayMessages.map((msg, idx) => {
        if (isCompressionSummaryMessage(msg)) return <ContextCompressionMarker key={msg.id} />;
        if (isCompressionContextOnlyMessage(msg)) return null;
        if (msg.role === "user") {
          return (
            <Fragment key={msg.id}>
              <UserMessage
                content={msg.content} files={msg.files}
                skillNames={msg.skill_names} isStreaming={isStreaming}
                onReload={onReload ? () => onReload(msg.id) : undefined}
                onEdit={onEdit ? (c) => onEdit(msg.id, c) : undefined}
                onClone={onCloneMessage ? () => onCloneMessage(msg.id) : undefined}
                onFileClick={onFileClick}
              />
              {artifactsAfter(msg.id)}
            </Fragment>
          );
        }
        if (msg.role === "assistant") {
          const isLast = idx === lastAssistantIdx && !isStreaming;
          return (
            <Fragment key={msg.id}>
              <SegmentedAssistantMessage
                msg={msg} onReload={onReload}
                onClone={onCloneMessage}
                onFilePreview={onFilePreview}
                projectPath={projectPath}
                tps={isLast ? tps : 0}
                totalElapsedMs={isLast ? totalElapsedMs : 0}
                workDurationMs={msg.work_duration_ms}
                liveCheckpoint={isStreaming && (
                  msg.stream_run_id === streamRunId || msg.is_stream_checkpoint === true
                )}
              />
              {artifactsAfter(msg.id)}
            </Fragment>
          );
        }
        return null;
      })}

      {isStreaming && (
        <StreamToolTimeline
          completedSegments={completedSegments}
          currentContent={currentContent}
          currentContentPhase={currentContentPhase}
          currentThinking={currentThinking}
          currentTools={currentTools}
          activeStreamItem={activeStreamItem}
          streamStartedAt={streamStartedAt}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      )}
      {queuedUserMessages.map((message) => (
        <UserMessage
          key={message.id}
          content={message.content}
          files={message.files}
          skillNames={message.skill_names}
          isStreaming
          onFileClick={onFileClick}
        />
      ))}
      {planPreview && <PlanPreviewBubble plan={planPreview} />}
      {isStreaming && !isCompressing && streamStartedAt != null && (
        <LoadingIndicator startedAt={streamStartedAt} liveTokenCount={liveTokenCount} />
      )}

      {isCompressing && <CompressionIndicator />}

    </>
  );
}

export const SegmentedAssistantMessage = memo(function SegmentedAssistantMessage({
  msg, onReload, onClone, onFilePreview, tps, totalElapsedMs, workDurationMs, projectPath,
  liveCheckpoint = false,
}: {
  msg: AgentMessage; onReload?: (id: string) => void; onFilePreview?: (path: string) => void;
  onClone?: (id: string) => void; tps: number; totalElapsedMs: number;
  workDurationMs?: number; projectPath?: string;
  liveCheckpoint?: boolean;
}) {
  if (msg.segments && msg.segments.length > 0) {
    return (
      <>
        <SavedToolTimeline
          messageId={msg.id}
          segments={msg.segments}
          tokens={msg.tokens}
          tps={tps}
          totalElapsedMs={workDurationMs ?? totalElapsedMs}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
          liveCheckpoint={liveCheckpoint}
          onClone={() => onClone?.(msg.id)}
        />
      </>
    );
  }
  return (
    <>
      <AssistantMessage
        content={msg.content} thinking={msg.thinking}
        toolActivities={msg.tool_activities}
        projectPath={projectPath}
        onReload={onReload ? () => onReload(msg.id) : undefined}
        onClone={onClone ? () => onClone(msg.id) : undefined}
        tokens={msg.tokens}
        tps={tps}
      />
    </>
  );
});
