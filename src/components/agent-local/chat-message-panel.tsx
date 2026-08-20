import { MessageList } from "./message-list";
import type { useAgentChat } from "@/hooks/use-agent-chat";
import type { useChatViewRuntime } from "@/hooks/use-chat-view-runtime";
import type { SubagentInfo } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

export interface ChatMessagePanelProps {
  chat: ReturnType<typeof useAgentChat>;
  runtime: ReturnType<typeof useChatViewRuntime>;
  projectPath?: string;
  knownSubagents: SubagentInfo[];
  cloneEnabled: boolean;
  requestClone: (messageId: string) => void;
  onFilePreviewPath?: (target: string | FileOperation) => void;
  onOpenSubagent?: (sessionId: string) => void;
  readOnly: boolean;
}

export function ChatMessagePanel({
  chat,
  runtime,
  projectPath,
  knownSubagents,
  cloneEnabled,
  requestClone,
  onFilePreviewPath,
  onOpenSubagent,
  readOnly,
}: ChatMessagePanelProps) {
  return (
    <MessageList
      messages={chat.messages}
      queuedUserMessages={chat.queuedUserMessages}
      completedSegments={chat.completedSegments}
      currentContent={chat.currentContent}
      currentContentPhase={chat.currentContentPhase}
      currentThinking={chat.currentThinking}
      currentTools={chat.currentTools}
      activeStreamItem={chat.activeStreamItem}
      isStreaming={chat.isStreaming}
      isCompressing={chat.isCompressing}
      tps={chat.tps}
      tpsEstimated={chat.tpsEstimated}
      totalElapsedMs={chat.totalElapsedMs}
      segmentStartedAt={chat.streamStartedAt}
      liveTokenCount={chat.liveTokenCount}
      streamRunId={chat.streamRunId}
      planPreview={chat.planPreview}
      onReload={readOnly ? undefined : runtime.handleReload}
      onEdit={readOnly ? undefined : runtime.handleEdit}
      onCloneMessage={readOnly || !cloneEnabled ? undefined : requestClone}
      onFileClick={runtime.handleFileClick}
      onFilePreview={onFilePreviewPath}
      projectPath={projectPath}
      onFileReview={onFilePreviewPath}
      knownSubagents={knownSubagents}
      onOpenSubagent={onOpenSubagent}
    />
  );
}
