import { useState } from "react";
import type { ReactNode } from "react";
import { AssistantMessage } from "./assistant-message";
import { BranchBubble } from "./branch-bubble";
import { ThinkingSection } from "./thinking-section";
import { ToolBubble, SavedToolBubble } from "./tool-bubble";
import { extractBranchActivity } from "./message-tool-aggregation";
import { WorkStreamSummary } from "./work-stream-summary";
import { hasNarrative } from "./message-tool-blocks";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ActiveStreamItem } from "@/hooks/active-stream-item";
import type { SavedSegment, TokenPhase } from "@/types/agent";
import type { ToolTimelineBlock } from "./message-tool-blocks";

export function TimelineLiveBlock({
  block,
  activeStreamItem,
  activeTools,
  onFilePreview,
  projectPath,
}: {
  block: ToolTimelineBlock<ToolActivity>;
  activeStreamItem: ActiveStreamItem;
  activeTools?: ToolActivity[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  return (
    <div>
      {block.isCurrent && hasNarrative(block) ? (
        <AssistantMessage
          content={block.content ?? ""}
          thinking={block.thinking}
          isStreaming
          thinkingActive={activeStreamItem?.kind === "thinking"}
          variant="trace"
        />
      ) : (
        <TimelineNarrative block={block} />
      )}
      {block.tools.length > 0 && (
        <ToolBubble
          tools={block.tools}
          activeTools={activeTools}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      )}
    </div>
  );
}

export function TimelineWorkBlock<T extends ToolActivity | SavedSegment["tools"][number]>({
  block,
  bubbleKind,
  activeTools,
  onFilePreview,
  projectPath,
}: {
  block: ToolTimelineBlock<T>;
  bubbleKind: "stream" | "saved";
  activeTools?: ToolActivity[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const branchActivity = bubbleKind === "saved"
    ? extractBranchActivity(block.tools as SavedSegment["tools"])
    : null;
  return (
    <div>
      <TimelineNarrative block={block} />
      {block.tools.length > 0 && (
        bubbleKind === "stream"
          ? (
            <ToolBubble
              tools={block.tools as ToolActivity[]}
              activeTools={activeTools}
              onFilePreview={onFilePreview}
              projectPath={projectPath}
            />
          )
          : <SavedToolBubble tools={block.tools as SavedSegment["tools"]} onFilePreview={onFilePreview} projectPath={projectPath} />
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

function TimelineNarrative<T>({ block }: { block: ToolTimelineBlock<T> }) {
  return (
    <>
      {block.thinking && <ThinkingSection content={block.thinking} />}
      {block.content && <AssistantMessage content={block.content} showActions={false} variant="trace" />}
    </>
  );
}

export function isFinalStreamPhase<T>(
  blocks: Array<ToolTimelineBlock<T>>,
  currentContent: string,
  currentContentPhase?: TokenPhase,
): number {
  if (currentContentPhase !== "final") return -1;
  if (!currentContent) return -1;
  const lastIndex = blocks.length - 1;
  const last = blocks[lastIndex];
  if (!last?.isCurrent || !last.content) return -1;
  const hasPreviousWork = blocks.slice(0, lastIndex).some(hasWorkContent);
  return hasPreviousWork ? lastIndex : -1;
}

export function savedWorkBlocks<T>(
  blocks: Array<ToolTimelineBlock<T>>,
  finalIndex: number,
): Array<ToolTimelineBlock<T>> {
  const finalBlock = blocks[finalIndex];
  const workBlocks = blocks.filter((_, index) => index !== finalIndex);
  if (finalBlock.thinking || finalBlock.tools.length > 0) {
    workBlocks.push({
      thinking: finalBlock.thinking,
      tools: finalBlock.tools,
      content: "",
      isCurrent: false,
      phase: "work",
    });
  }
  return workBlocks;
}

export function hasWorkContent<T>(block: ToolTimelineBlock<T>): boolean {
  return !!block.thinking || !!block.content || block.tools.length > 0;
}

export function LiveWorkStreamSummary({
  children,
  startedAt,
}: {
  children: ReactNode;
  startedAt: number | null;
}) {
  const [durationMs] = useState(() => (startedAt ? Date.now() - startedAt : undefined));
  return <WorkStreamSummary durationMs={durationMs}>{children}</WorkStreamSummary>;
}
