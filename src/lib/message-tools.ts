import type { AgentMessage, ToolActivityRecord } from "@/types/agent";

export function toolsFromMessage(message: AgentMessage): ToolActivityRecord[] {
  if (message.segments?.length) {
    const segmented = message.segments.flatMap((segment) => segment.tools);
    if (segmented.length > 0) return segmented;
  }
  return message.tool_activities ?? [];
}
