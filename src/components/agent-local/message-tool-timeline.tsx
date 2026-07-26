import { AssistantMessage } from "./assistant-message";
import { buildToolTimelineBlocks } from "./message-tool-blocks";
import {
  isFinalStreamPhase,
  LiveWorkStreamSummary,
  savedWorkBlocks,
  TimelineLiveBlock,
  TimelineWorkBlock,
} from "./message-tool-timeline-render";
export { SavedToolTimeline } from "./saved-tool-timeline";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import type { ActiveStreamItem } from "@/hooks/active-stream-item";
import type { TokenPhase } from "@/types/agent";

interface StreamToolTimelineProps {
  completedSegments: StreamSegment[];
  currentContent: string;
  currentContentPhase?: TokenPhase;
  currentThinking: string;
  currentTools: ToolActivity[];
  activeStreamItem?: ActiveStreamItem;
  streamStartedAt: number | null;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}

export function StreamToolTimeline({
  completedSegments,
  currentContent,
  currentContentPhase,
  currentThinking,
  currentTools,
  activeStreamItem = null,
  streamStartedAt,
  onFilePreview,
  projectPath,
}: StreamToolTimelineProps) {
  const activeTools = activeStreamItem?.kind === "tools"
    ? activeStreamItem.toolIndices.flatMap((index) => currentTools[index] ? [currentTools[index]] : [])
    : [];
  const segments = [
    ...completedSegments,
    ...(currentContent || currentThinking || currentTools.length > 0
      ? [{
        thinking: currentThinking,
        content: currentContent,
        tools: currentTools,
        isCurrent: true,
        phase: currentContentPhase,
      }]
      : []),
  ];
  const blocks = buildToolTimelineBlocks(segments);
  const finalIndex = isFinalStreamPhase(blocks, currentContent, currentContentPhase);
  if (finalIndex >= 0) {
    const finalBlock = blocks[finalIndex];
    const workBlocks = savedWorkBlocks(blocks, finalIndex);
    return (
      <>
        <LiveWorkStreamSummary startedAt={streamStartedAt}>
          {workBlocks.map((block, index) => (
            <TimelineWorkBlock
              key={`stream-work-${index}`}
              block={block}
              bubbleKind="stream"
              activeTools={activeTools}
              onFilePreview={onFilePreview}
              projectPath={projectPath}
            />
          ))}
        </LiveWorkStreamSummary>
        <AssistantMessage
          content={finalBlock.content ?? ""}
          isStreaming
        />
      </>
    );
  }

  return (
    <>
      {blocks.map((block, index) => (
        <TimelineLiveBlock
          key={`stream-block-${index}`}
          block={block}
          activeStreamItem={activeStreamItem}
          activeTools={activeTools}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      ))}
    </>
  );
}
