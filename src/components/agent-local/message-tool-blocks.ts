export interface ToolTimelineSegment<T> {
  thinking?: string;
  content?: string;
  tools: T[];
  isCurrent?: boolean;
  phase?: "work" | "final";
}

export interface ToolTimelineBlock<T> {
  thinking?: string;
  content?: string;
  tools: T[];
  isCurrent: boolean;
  phase?: "work" | "final";
}

export function buildToolTimelineBlocks<T>(
  segments: Array<ToolTimelineSegment<T>>,
): Array<ToolTimelineBlock<T>> {
  const blocks: Array<ToolTimelineBlock<T>> = [];
  for (const segment of segments) {
    if (hasNarrative(segment)) {
      const block: ToolTimelineBlock<T> = {
        thinking: segment.thinking,
        content: segment.content,
        tools: [...segment.tools],
        isCurrent: segment.isCurrent === true,
      };
      if (segment.phase) block.phase = segment.phase;
      blocks.push(block);
      continue;
    }
    if (segment.tools.length === 0) continue;
    const last = blocks[blocks.length - 1];
    if (last) {
      last.tools.push(...segment.tools);
    } else {
      const block: ToolTimelineBlock<T> = { tools: [...segment.tools], isCurrent: false };
      if (segment.phase) block.phase = segment.phase;
      blocks.push(block);
    }
  }
  return blocks;
}

export function hasNarrative(segment: { thinking?: string; content?: string }): boolean {
  return !!segment.thinking || !!segment.content;
}
