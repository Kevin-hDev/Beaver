import { AssistantMessage } from "./assistant-message";
import { BranchBubble } from "./branch-bubble";
import { ThinkingSection } from "./thinking-section";
import { SavedToolBubble } from "./tool-bubble";
import { buildToolTimelineBlocks } from "./message-tool-blocks";
import { extractBranchActivity } from "./message-tool-aggregation";
import {
  hasWorkContent,
  savedWorkBlocks,
  TimelineWorkBlock,
} from "./message-tool-timeline-render";
import { WorkStreamSummary } from "./work-stream-summary";
import type { SavedSegment } from "@/types/agent";

interface SavedToolTimelineProps {
  messageId: string;
  segments: SavedSegment[];
  tokens?: number;
  tps: number;
  totalElapsedMs: number;
  onFilePreview?: (path: string) => void;
  onClone?: () => void;
  projectPath?: string;
  liveCheckpoint?: boolean;
}

export function SavedToolTimeline({
  messageId, segments, tokens, tps, totalElapsedMs, onFilePreview, onClone,
  projectPath, liveCheckpoint = false,
}: SavedToolTimelineProps) {
  const blocks = buildToolTimelineBlocks(segments);
  if (liveCheckpoint) {
    return (
      <>
        {blocks.map((block, index) => (
          <TimelineWorkBlock
            key={`${messageId}-live-${index}`}
            block={block}
            bubbleKind="saved"
            onFilePreview={onFilePreview}
            projectPath={projectPath}
          />
        ))}
      </>
    );
  }
  const hasPhase = blocks.some((block) => !!block.phase);
  const explicitFinalIndex = findLastIndex(
    blocks,
    (block) => block.phase === "final" && !!block.content,
  );
  const lastTextIndex = explicitFinalIndex >= 0
    ? explicitFinalIndex
    : hasPhase ? -1 : findLastIndex(blocks, (block) => !!block.content);
  if (lastTextIndex >= 0) {
    const finalBlock = blocks[lastTextIndex];
    const workBlocks = savedWorkBlocks(blocks, lastTextIndex);
    if (workBlocks.some(hasWorkContent)) {
      return (
        <>
          <WorkStreamSummary durationMs={totalElapsedMs > 0 ? totalElapsedMs : undefined}>
            {workBlocks.map((block, index) => (
              <TimelineWorkBlock
                key={`${messageId}-work-${index}`}
                block={block}
                bubbleKind="saved"
                onFilePreview={onFilePreview}
                projectPath={projectPath}
              />
            ))}
          </WorkStreamSummary>
          <AssistantMessage content={finalBlock.content ?? ""} tokens={tokens} tps={tps} onClone={onClone} />
        </>
      );
    }
    return (
      <AssistantMessage
        content={finalBlock.content ?? ""}
        tokens={tokens}
        tps={tps}
        totalElapsedMs={totalElapsedMs}
        onClone={onClone}
      />
    );
  }
  if (hasPhase && blocks.some(hasWorkContent)) {
    return (
      <WorkStreamSummary durationMs={totalElapsedMs > 0 ? totalElapsedMs : undefined}>
        {blocks.map((block, index) => (
          <TimelineWorkBlock
            key={`${messageId}-work-only-${index}`}
            block={block}
            bubbleKind="saved"
            onFilePreview={onFilePreview}
            projectPath={projectPath}
          />
        ))}
      </WorkStreamSummary>
    );
  }
  return (
    <>
      {blocks.map((block, index) => (
        <SavedBlock
          key={`${messageId}-block-${index}`}
          block={block}
          showStats={index === lastTextIndex}
          tokens={tokens}
          tps={tps}
          totalElapsedMs={totalElapsedMs}
          onClone={onClone}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      ))}
    </>
  );
}

function SavedBlock({
  block, showStats, tokens, tps, totalElapsedMs, onClone, onFilePreview, projectPath,
}: {
  block: ReturnType<typeof buildToolTimelineBlocks<SavedSegment["tools"][number]>>[number];
  showStats: boolean;
  tokens?: number;
  tps: number;
  totalElapsedMs: number;
  onClone?: () => void;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const branchActivity = extractBranchActivity(block.tools);
  return (
    <div>
      {block.thinking && <ThinkingSection content={block.thinking} />}
      {block.content && (
        <AssistantMessage
          content={block.content}
          tokens={showStats ? tokens : undefined}
          tps={showStats ? tps : undefined}
          totalElapsedMs={showStats ? totalElapsedMs : undefined}
          onClone={showStats ? onClone : undefined}
        />
      )}
      {block.tools.length > 0 && (
        <SavedToolBubble tools={block.tools} onFilePreview={onFilePreview} projectPath={projectPath} />
      )}
      {branchActivity && (
        <BranchBubble
          action={branchActivity.action}
          branchName={branchActivity.branchName}
          path={branchActivity.path}
        />
      )}
    </div>
  );
}

function findLastIndex<T>(items: T[], predicate: (item: T) => boolean): number {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    if (predicate(items[index])) return index;
  }
  return -1;
}
