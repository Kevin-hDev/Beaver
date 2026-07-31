import type { ManagedStreamState } from "./agent-chat-stream-types";
import { pendingToolIndices, toolItems } from "./active-stream-item";

export interface ToolOutputData {
  toolCallIndex: number;
  content: string;
  elapsedMs: number;
}

export function applyToolOutput(state: ManagedStreamState, data: ToolOutputData) {
  const directIndex = data.toolCallIndex;
  const isShell = (index: number) => {
    const name = state.currentTools[index]?.name;
    return name === "bash" || name === "bash_write";
  };
  const index = directIndex >= 0 && isShell(directIndex)
    ? directIndex
    : state.currentTools.findIndex((tool) =>
        (tool.name === "bash" || tool.name === "bash_write") && tool.result === undefined);
  if (index < 0) return;
  const tools = [...state.currentTools];
  tools[index] = {
    ...tools[index],
    liveOutput: data.content,
    liveElapsedMs: data.elapsedMs,
  };
  state.currentTools = tools;
  state.activeStreamItem = toolItems(pendingToolIndices(tools));
}
